use anyhow::{Result, bail};
use ratatui::{Terminal, backend::Backend};
use std::sync::mpsc as std_chan;

use crate::{
    config::Config,
    event_handler::{self, Event},
    model::{
        connection::Connection,
        musing::{MusingState, MusingStateDelta},
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
    Queue,
    Library,
    // Playlists,
    // FileTree,
}

#[derive(Debug)]
pub struct App {
    pub connection: Connection,
    pub app_state: AppState,
    pub screen: Screen,
    pub musing_state: MusingState,
}

impl App {
    pub fn try_new(config: Config) -> Result<Self> {
        let Config { port } = config;
        let mut connection = Connection::try_new(port)?;
        let app_state = AppState::default();
        let screen = Screen::default();
        let musing_state = MusingState::default();

        Ok(Self {
            connection,
            app_state,
            screen,
            musing_state,
        })
    }

    pub fn run(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<()> {
        let (tx_event, rx_event) = std_chan::channel();
        event_handler::run(tx_event);
        loop {
            match rx_event.recv() {
                Ok(event) => match event {
                    Event::Keypress(ev) => {
                        // let update = update::translate_key_event(ev);
                        // update::update(self, update);
                    }
                    Event::Refresh => {
                        if let Ok(delta_json) = self.connection.state_delta()
                            && let Ok(delta) = MusingStateDelta::try_from(delta_json)
                        {
                            update::update_musing_state(&mut self.musing_state, delta);
                        }
                    }
                },
                Err(_) => bail!("event handler crashed"),
            }
            terminal.draw(|frame| view::render(&self, frame))?;
            if let AppState::Done = self.app_state {
                break;
            }
        }

        Ok(())
    }
}
