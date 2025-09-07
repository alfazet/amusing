use anyhow::Result;
use ratatui::widgets::{ListState, TableState};
use std::collections::HashMap;

use crate::model::common::Scroll;

#[derive(Debug, Default)]
pub enum ActivePart {
    #[default]
    Groups, // "lhs" of the view
    Child(usize), // "rhs" of the view, which child is active
}

#[derive(Debug, Default)]
pub struct SongGroup {
    pub metadata: Vec<HashMap<String, String>>,
    pub paths: Vec<String>,
}

#[derive(Debug, Default)]
pub struct LibraryChildState {
    pub state: ListState,
    pub id_comb: Vec<String>, // the combination of tags that identifies these songs
    pub group: SongGroup,
}

#[derive(Debug)]
pub struct LibraryState {
    pub state: TableState,
    pub active_part: ActivePart,
    pub group_by_tags: Vec<String>,
    pub children_tags: Vec<String>,
    pub children: Vec<LibraryChildState>, // grouped collections of songs
}

impl SongGroup {
    fn pair_values(
        keys: &[String],
        values: &[Vec<Option<String>>],
    ) -> Vec<HashMap<String, String>> {
        let mut res = Vec::new();
        for values_inner in values.iter() {
            let mut map = HashMap::new();
            for (key, value) in keys.iter().zip(values_inner) {
                if let Some(value) = value {
                    map.insert(key.clone(), value.clone());
                }
            }
            res.push(map);
        }

        res
    }

    pub fn new(
        metadata_keys: &[String],
        metadata_values: &[Vec<Option<String>>],
        paths: &[String],
    ) -> Self {
        let metadata = Self::pair_values(metadata_keys, metadata_values);
        Self {
            metadata,
            paths: paths.to_vec(),
        }
    }

    pub fn add_songs(
        &mut self,
        metadata_keys: &[String],
        metadata_values: &[Vec<Option<String>>],
        paths: &[String],
    ) {
        // let metadata_pairs: Vec<_> = metadata_keys.into_iter().zip(metadata_values.into_iter().filter_map(|val| val)).collect();
    }
}

impl Default for LibraryState {
    fn default() -> Self {
        Self {
            state: TableState::default(),
            active_part: ActivePart::default(),
            group_by_tags: vec!["albumartist".into(), "album".into()],
            children_tags: vec!["tracknumber".into(), "tracktitle".into()],
            children: Vec::new(),
        }
    }
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
    pub fn update(&mut self, grouped_songs: HashMap<Vec<String>, SongGroup>) {
        self.children.clear();
        for (id_comb, group) in grouped_songs {
            let child = LibraryChildState {
                state: ListState::default(),
                id_comb,
                group,
            };
            self.children.push(child);
        }
        self.children
            .sort_unstable_by(|lhs, rhs| (lhs.id_comb).cmp(&rhs.id_comb));
        self.state.select_first();
    }

    pub fn selected_child(&self) -> Option<&LibraryChildState> {
        self.state.selected().map(|i| &self.children[i])
    }

    pub fn selected_child_mut(&mut self) -> Option<&mut LibraryChildState> {
        self.state.selected().map(|i| &mut self.children[i])
    }
}
