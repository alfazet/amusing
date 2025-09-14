use ratatui::widgets::TableState;
use std::collections::HashMap;

use crate::model::{
    common::Scroll,
    search::{Search, SearchState},
};

#[derive(Debug)]
pub struct QueueState {
    pub state: TableState,
    pub metadata: Vec<HashMap<String, String>>,
    pub displayed_tags: Vec<String>, // tags to be displayed to the user
    pub search: Search,
}

impl Default for QueueState {
    fn default() -> Self {
        Self {
            state: TableState::default(),
            metadata: Vec::new(),
            displayed_tags: vec!["tracktitle".into(), "artist".into(), "album".into()],
            search: Search::default(),
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
        self.state
            .select(Some(self.metadata.len().saturating_sub(1)));
    }
}

impl QueueState {
    pub fn search_on(&mut self) {
        self.scroll_to_top();
        self.search.on(self.metadata_to_repr());
    }

    pub fn unordered_selected(&self) -> Option<usize> {
        self.state.selected().map(|i| self.search.real_i(i))
    }

    pub fn metadata_to_repr(&self) -> Vec<String> {
        self.metadata
            .iter()
            .map(|m| {
                let mut repr = String::new();
                for tag in &self.displayed_tags {
                    if let Some(value) = m.get(tag) {
                        repr += value;
                    }
                }

                unidecode::unidecode(&repr)
            })
            .collect()
    }

    pub fn ordered_metadata(&self) -> Vec<&HashMap<String, String>> {
        let search = &self.search;
        match search.state {
            SearchState::Off => self.metadata.iter().collect(),
            _ => {
                let mut ordered = Vec::with_capacity(self.metadata.len());
                // clone not to hold up the guard
                let order = { search.result.read().unwrap().clone() };
                for m in order.iter().filter_map(|&i| self.metadata.get(i)) {
                    ordered.push(m);
                }

                ordered
            }
        }
    }
}
