use anyhow::{Result, bail};
use ratatui::{Terminal, backend::Backend};
use std::{collections::HashMap, sync::mpsc as std_chan};

use crate::{
    config::Config,
    event_handler::{self, Event},
    model::{
        connection::Connection,
        musing::{MusingState, MusingStateDelta},
        queue::QueueState,
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
    // Playlists,
    // FileTree,
}

#[derive(Debug)]
pub struct App {
    pub connection: Connection,
    pub app_state: AppState,
    pub screen: Screen,
    pub musing_state: MusingState,
    pub queue_state: QueueState,
    pub metadata: Vec<HashMap<String, String>>,
    // pub status_msg: Option<String>,
}

impl App {
    pub fn try_new(config: Config) -> Result<Self> {
        let Config { port } = config;
        let connection = Connection::try_new(port)?;
        let app_state = AppState::default();
        let screen = Screen::default();
        let musing_state = MusingState::default();
        let queue_state = QueueState::default();
        let metadata = Vec::new();
        // let status_msg = None;

        Ok(Self {
            connection,
            app_state,
            screen,
            musing_state,
            queue_state,
            metadata,
            // status_msg,
        })
    }

    pub fn run(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<()> {
        let (tx_event, rx_event) = std_chan::channel();
        event_handler::run(tx_event);
        loop {
            match rx_event.recv() {
                Ok(event) => match event {
                    Event::Keypress(ev) => {
                        if let Some(msg) = update::translate_key_event(self, ev) {
                            update::update_app(self, msg);
                        }
                    }
                    Event::Refresh => {
                        if let Ok(delta_json) = self.connection.state_delta()
                            && let Ok(delta) = MusingStateDelta::try_from(delta_json)
                        {
                            update::main_update(self, delta);
                        }
                    }
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
