use ratatui::widgets::TableState;

use crate::model::{
    common::{Scroll, SongGroup},
    search::{Search, SearchState},
};

#[derive(Debug, Default)]
pub struct QueueState {
    pub state: TableState,
    pub group: SongGroup,
    pub queue_tags: Vec<String>, // tags to be displayed to the user
    pub search: Search,
}

impl Scroll for QueueState {
    fn scroll(&mut self, delta: i32) {
        if self.group.is_empty() {
            return;
        }
        let u_delta = delta.unsigned_abs() as usize;
        let n_rows = self.group.len();
        match self.state.selected() {
            Some(r) => {
                if delta < 0 {
                    if r >= u_delta {
                        self.state.scroll_up_by(u_delta as u16);
                    } else {
                        self.state.select(Some(n_rows - (u_delta - r)));
                    }
                } else if r + u_delta < n_rows {
                    self.state.scroll_down_by(u_delta as u16);
                } else {
                    self.state.select(Some(u_delta - (n_rows - r)));
                }
            }
            None => self.state.select_first(),
        };
    }

    fn scroll_to_top(&mut self) {
        if self.group.is_empty() {
            return;
        }
        self.state.select_first();
    }

    fn scroll_to_bottom(&mut self) {
        if self.group.is_empty() {
            return;
        }
        self.state.select(Some(self.group.len().saturating_sub(1)));
    }
}

impl QueueState {
    pub fn new(queue_tags: Vec<String>) -> Self {
        Self {
            state: TableState::default(),
            group: SongGroup::default(),
            queue_tags,
            search: Search::default(),
        }
    }

    pub fn search_on(&mut self) {
        self.scroll_to_top();
        self.search.on(self.metadata_to_repr());
    }

    pub fn unordered_selected(&self) -> Option<usize> {
        self.state.selected().map(|i| self.search.real_i(i))
    }

    pub fn metadata_to_repr(&self) -> Vec<String> {
        self.group
            .metadata
            .iter()
            .map(|m| {
                let mut repr = String::new();
                for tag in &self.queue_tags {
                    if let Some(value) = m.get(tag) {
                        repr += value;
                        repr.push(' ');
                    }
                }

                unidecode::unidecode(&repr)
            })
            .collect()
    }

    pub fn ordered_group(&self) -> SongGroup {
        let search = &self.search;
        match search.state {
            SearchState::Off => self.group.clone(),
            _ => {
                // clone not to hold up the guard
                let order = { search.result.read().unwrap().clone() };
                self.group.new_ordered(&order)
            }
        }
    }
}
