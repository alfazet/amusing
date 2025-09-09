use anyhow::Result;
use ratatui::widgets::TableState;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, mpsc as std_chan},
};
use tui_input::Input as TuiInput;

use crate::model::{
    common::{Scroll, Search},
    search::{self, SearchMessage, SearchState},
};

#[derive(Debug)]
pub struct QueueState {
    pub state: TableState,
    pub metadata: Vec<HashMap<String, String>>,
    pub displayed_tags: Vec<String>, // tags to be displayed to the user
    pub search: Option<SearchState>,
}

impl Default for QueueState {
    fn default() -> Self {
        Self {
            state: TableState::default(),
            metadata: Vec::new(),
            displayed_tags: vec!["tracktitle".into(), "artist".into(), "album".into()],
            search: None,
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

impl Search for QueueState {
    fn search_on(&mut self) {
        let (tx, rx) = std_chan::channel();
        let result = Arc::new(RwLock::new((0..self.metadata.len()).collect()));
        let list = self.metadata_to_repr();
        self.search = Some(SearchState {
            tx,
            result: Arc::clone(&result),
            input: TuiInput::default(),
            active: true,
        });
        search::run(list, rx, result);
    }

    fn search_off(&mut self) {
        let _ = self.search.take();
        // the sender gets dropped => searching thread finishes
    }

    // confirm the search query
    fn search_idle(&mut self) {
        if let Some(search) = &mut self.search {
            search.active = false;
        }
    }

    // send a new pattern to the search thread
    fn search_pattern_update(&mut self, pattern: String) {
        if let Some(search) = &self.search {
            let _ = search.tx.send(SearchMessage::NewPattern(pattern));
        }
    }

    // send a new list to the search thread
    fn search_list_update(&mut self, list: Vec<String>) {
        if let Some(search) = &self.search {
            let _ = search.tx.send(SearchMessage::NewList(list));
        }
    }
}

impl QueueState {
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

    // translates the "view" i into the "actual" i
    pub fn real_i(&self, i: usize) -> usize {
        match &self.search {
            Some(search) => {
                let order = search.result.read().unwrap();
                (*order).get(i).copied().unwrap_or_default()
            }
            None => i,
        }
    }

    // returns the currently selected index, but taking into account
    // the fact that the view may be sorted in a different order
    pub fn unordered_selected(&self) -> Option<usize> {
        self.state.selected().map(|i| self.real_i(i))
    }

    pub fn ordered_metadata(&self) -> Vec<&HashMap<String, String>> {
        match &self.search {
            Some(search) => {
                let mut ordered = Vec::with_capacity(self.metadata.len());
                // clone not to hold up the guard
                let order = { search.result.read().unwrap().clone() };
                for m in order.iter().filter_map(|&i| self.metadata.get(i)) {
                    ordered.push(m);
                }
                // let max_i = order.into_iter().max().unwrap_or_default();
                // add any items that weren't there back when we started the search
                // if max_i < self.metadata.len().saturating_sub(1) {
                //     for m in &self.metadata[(max_i + 1)..] {
                //         ordered.push(m);
                //     }
                // }

                ordered
            }
            None => self.metadata.iter().collect(),
        }
    }
}
