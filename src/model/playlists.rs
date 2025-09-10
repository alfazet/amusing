use anyhow::Result;
use ratatui::widgets::{ListState, TableState};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, mpsc as std_chan},
};
use tui_input::Input as TuiInput;

use crate::model::{
    common::{FocusedPart, Scroll, SongGroup},
    search::{self, Search, SearchMessage},
};

#[derive(Debug, Default)]
pub struct PlaylistsChildState {
    pub state: TableState,
    pub name: String,
    pub group: SongGroup,
}
