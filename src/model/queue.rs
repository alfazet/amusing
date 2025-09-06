use anyhow::Result;
use ratatui::widgets::TableState;
use std::collections::HashMap;

use crate::model::common::Scroll;

#[derive(Debug, Default)]
pub struct QueueState {
    pub state: TableState,
    pub metadata: Vec<HashMap<String, String>>,
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
        self.state.select_last();
    }
}
