use anyhow::Result;
use ratatui::widgets::{ListState, TableState};
use std::collections::HashMap;

use crate::model::common::Scroll;

#[derive(Debug, Default)]
pub enum ActivePart {
    #[default]
    Groups,
    Child(usize),
}

#[derive(Debug, Default)]
pub struct LibraryChildState {
    pub state: ListState,
    pub id: Vec<String>, // the combination of tags that identifies these songs
    pub titles: Vec<String>,
}

#[derive(Debug, Default)]
pub struct LibraryState {
    pub state: TableState,
    pub active_part: ActivePart,
    pub children: Vec<LibraryChildState>, // sorted and grouped collections of songs
}

impl Scroll for LibraryState {
    fn scroll(&mut self, delta: i32) {
        let u_delta = delta.unsigned_abs() as usize;
        let n_rows = self.children.len();
        match self.state.selected() {
            Some(r) => {
                if delta < 0 {
                    if r >= u_delta {
                        self.state.scroll_up_by(u_delta as u16);
                    } else {
                        self.state.select(Some(n_rows - (u_delta - r)));
                    }
                } else {
                    if r + u_delta < n_rows {
                        self.state.scroll_down_by(u_delta as u16);
                    } else {
                        self.state.select(Some(u_delta - (n_rows - r)));
                    }
                }
            }
            None => self.state.select_first(),
        };
    }

    fn scroll_to_top(&mut self) {
        self.state.select_first();
    }

    fn scroll_to_bottom(&mut self) {
        self.state.select_last();
    }
}

impl LibraryState {
    pub fn update(&mut self, grouped_songs: HashMap<Vec<String>, Vec<String>>) {
        self.children.clear();
        for (id, titles) in grouped_songs {
            let child = LibraryChildState {
                state: ListState::default(),
                id,
                titles,
            };
            self.children.push(child);
        }
        self.children
            .sort_unstable_by(|lhs, rhs| (lhs.id).cmp(&rhs.id));
        self.state.select_first();
    }

    pub fn selected_child(&self) -> Option<&LibraryChildState> {
        self.state.selected().map(|i| &self.children[i])
    }

    pub fn selected_child_mut(&mut self) -> Option<&mut LibraryChildState> {
        self.state.selected().map(|i| &mut self.children[i])
    }
}
