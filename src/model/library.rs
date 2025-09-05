use anyhow::Result;
use ratatui::widgets::TableState;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct LibraryChildState {
    pub state: TableState,
    pub titles: Vec<String>,
}

#[derive(Debug, Default)]
pub struct LibraryState {
    pub state: TableState,
    pub children: HashMap<Vec<String>, LibraryChildState>, // grouped collections of songs
}

impl LibraryState {
    pub fn update(&mut self, grouped_songs: HashMap<Vec<String>, Vec<String>>) {
        for (id_comb, titles) in grouped_songs {
            let child = LibraryChildState {
                state: TableState::default(),
                titles,
            };
            self.children.insert(id_comb, child);
        }
    }
}
