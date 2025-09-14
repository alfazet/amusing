use anyhow::{Result, bail};
use ratatui::{Terminal, backend::Backend, crossterm::event::KeyEvent};
use std::sync::mpsc as std_chan;

use crate::{
    config::Config,
    event_handler::{self, Event},
    model::{
        connection::Connection, cover_art::CoverArtState, keybind::Keybind, library::LibraryState,
        musing::MusingState, queue::QueueState, theme::Theme,
    },
    update, view,
};

#[derive(Debug, Default)]
pub enum AppState {
    #[default]
    Running,
    Done,
}

#[derive(Debug, Default)]
pub enum Screen {
    #[default]
    Cover,
    Queue,
    Library,
}

#[derive(Debug)]
pub struct AppConfig {
    pub theme: Theme,
    pub keybind: Keybind,
    pub seek_step: i64,
    pub volume_step: i8,
    pub speed_step: i16,
}

#[derive(Debug)]
pub struct App {
    pub connection: Connection,
    pub app_state: AppState,
    pub screen: Screen,
    pub musing_state: MusingState,
    pub queue_state: QueueState,
    pub library_state: LibraryState,
    pub key_events: Vec<KeyEvent>,
    pub status_msg: Option<String>,
    pub config: AppConfig,
}

impl App {
    pub fn try_new(config: Config) -> Result<Self> {
        let Config {
            port,
            theme,
            keybind,
            seek_step,
            volume_step,
            speed_step,
            library_group_by,
        } = config;
        let connection = Connection::try_new(port)?;
        let app_state = AppState::default();
        let screen = Screen::default();
        let musing_state = MusingState::default();
        let queue_state = QueueState::default();
        let library_state = LibraryState::new(library_group_by);
        let key_events = Vec::new();
        let status_msg = None;
        let config = AppConfig {
            theme,
            keybind,
            seek_step,
            volume_step,
            speed_step,
        };

        Ok(Self {
            connection,
            app_state,
            screen,
            musing_state,
            queue_state,
            library_state,
            key_events,
            status_msg,
            config,
        })
    }

    pub fn run(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<()> {
        let (tx_event, rx_event) = std_chan::channel();
        let tx_resize = tx_event.clone();
        // clone tx_event and pass it to the picture thread
        let mut cover_art_state = CoverArtState::try_new(tx_resize)?;
        event_handler::run(tx_event);

        update::update_library(self)?;
        loop {
            match rx_event.recv() {
                Ok(event) => match event {
                    Event::Keypress(ev) => {
                        if let Some(msg) = update::translate_key_event(self, ev) {
                            let _ = self.status_msg.take();
                            if let Err(e) = update::update_app(self, msg) {
                                self.status_msg =
                                    Some(format!("couldn't connect to musing ({})", e));
                            }
                        }
                    }
                    Event::Refresh => {
                        if let Ok(delta) = self.connection.state_delta() {
                            update::update_state(self, delta, &mut cover_art_state);
                        }
                    }
                    Event::CoverArtResize(redraw) => {
                        update::update_cover_art(&mut cover_art_state, redraw?);
                    }
                },
                Err(_) => bail!("event handler crashed"),
            }
            terminal.draw(|frame| view::render(self, frame, &mut cover_art_state))?;
            if let AppState::Done = self.app_state {
                break;
            }
        }

        Ok(())
    }
}
