use anyhow::Result;
use ratatui::widgets::TableState;
use std::collections::HashMap;

use crate::model::common::{Scroll, Search, SearchState};

#[derive(Debug)]
pub struct QueueState {
    pub state: TableState,
    pub metadata: Vec<HashMap<String, String>>,
    pub displayed_tags: Vec<String>, // tags to be displayed to the user
    // pub search: SearchState,
    // at every search query change sort the queue by the fuzzy comparison score
    // we're comparing the pattern typed by the user with a string made out of 
    // concatenating all values of displayed_tags (in the correct order)
}

impl Default for QueueState {
    fn default() -> Self {
        Self {
            state: TableState::default(),
            metadata: Vec::new(),
            displayed_tags: vec!["tracktitle".into(), "artist".into(), "album".into()],
        }
    }
}

impl Scroll for QueueState {
    fn scroll(&mut self, delta: i32) {
        let u_delta = delta.unsigned_abs() as usize;
        let n_rows = self.metadata.len();
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
        self.state.select(Some(self.metadata.len() - 1));
    }
}
