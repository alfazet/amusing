use anyhow::{Result, bail};
use ratatui::{Terminal, backend::Backend, crossterm::event::KeyEvent};
use std::sync::mpsc as std_chan;

use crate::{
    config::Config,
    event_handler::{self, Event},
    model::{
        connection::{Connection, MusingRequest},
        cover_art::CoverArtState,
        keybind::Keybind,
        library::LibraryState,
        musing::MusingState,
        queue::QueueState,
        theme::Theme,
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

pub struct App {
    pub connection: Connection,
    pub app_state: AppState,
    pub screen: Screen,
    pub musing_state: MusingState,
    pub queue_state: QueueState,
    pub library_state: LibraryState,
    pub cover_art_state: CoverArtState,
    pub key_events: Vec<KeyEvent>,
    pub status_msg: Option<String>,
    pub config: AppConfig,
    tx: std_chan::Sender<Event>,
    rx: std_chan::Receiver<Event>,
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
        let (tx, rx) = std_chan::channel();
        let connection = Connection::try_new(port, tx.clone())?;
        let app_state = AppState::default();
        let screen = Screen::default();
        let musing_state = MusingState::default();
        let queue_state = QueueState::default();
        let library_state = LibraryState::new(library_group_by);
        let cover_art_state = CoverArtState::try_new(tx.clone())?;
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
            cover_art_state,
            key_events,
            status_msg,
            config,
            tx,
            rx,
        })
    }

    pub fn run(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<()> {
        update::update_library(self);
        // TODO: move cover_art_state to self because now it should be easy
        // let mut cover_art_state = CoverArtState::try_new(self.tx.clone())?;
        event_handler::run(self.tx.clone());

        loop {
            // we get events from 4 sources: key presses, the automatic refresh, cover art
            // resizing and the network connection
            match self.rx.recv() {
                Ok(event) => match event {
                    Event::Keypress(ev) => {
                        if let Some(msg) = update::translate_key_event(self, ev) {
                            let _ = self.status_msg.take();
                            update::update_on_message(self, msg);
                        }
                    }
                    Event::CoverArtResize(redraw) => {
                        update::update_cover_art(self, redraw?)
                    }
                    Event::MusingResponse(response) => {
                        update::update_on_response(self, response)
                    }
                    Event::Refresh => self.connection.send(MusingRequest::StateDelta),
                },
                Err(_) => bail!("event handler crashed"),
            }
            terminal.draw(|frame| view::render(self, frame))?;
            if let AppState::Done = self.app_state {
                break;
            }
        }

        Ok(())
    }
}
