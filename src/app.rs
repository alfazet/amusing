use anyhow::Result;
use ratatui::{Terminal, backend::Backend};

use crate::{
    config::Config,
    model::{connection::Connection, musing::MusingState},
};

// move these things to model/...
#[derive(Debug, Default)]
enum AppState {
    #[default]
    Running,
    Done,
}

#[derive(Debug, Default)]
enum Screen {
    #[default]
    Queue,
    Library,
    // Playlists,
    // FileTree,
}

#[derive(Debug)]
pub struct App {
    connection: Connection,
    app_state: AppState,
    screen: Screen,
    musing_state: MusingState,
}

impl App {
    pub fn try_new(config: Config) -> Result<Self> {
        let Config { port } = config;
        let mut connection = Connection::try_new(port)?;
        let app_state = AppState::default();
        let screen = Screen::default();
        let musing_state = connection.state()?;

        Ok(Self {
            connection,
            app_state,
            screen,
            musing_state,
        })
    }

    pub fn run(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<()> {
        self.musing_state = self.connection.state()?;
        println!("{:?}", self.musing_state);

        Ok(())
    }
}
