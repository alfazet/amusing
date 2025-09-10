use anyhow::Result;
use ratatui::widgets::{ListState, TableState};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, mpsc as std_chan},
};
use tui_input::Input as TuiInput;

use crate::model::{
    common::{FocusedPart, Scroll, SongGroup},
    search::{self, Search, SearchMessage, SearchState},
};

#[derive(Debug, Default)]
pub struct LibraryChildState {
    pub state: TableState,
    pub id_comb: Vec<String>, // the combination of tags that identifies these songs
    pub group: SongGroup,
    pub search: Search,
}

#[derive(Debug)]
pub struct LibraryState {
    pub state: TableState,
    pub focused_part: FocusedPart,
    pub group_by_tags: Vec<String>,
    pub children_tags: Vec<String>,
    pub children: Vec<LibraryChildState>, // grouped collections of songs
    pub search: Search,
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
        self.metadata
            .extend(Self::pair_values(metadata_keys, metadata_values));
        self.paths.extend_from_slice(paths);
    }
}

impl LibraryChildState {
    pub fn search_on(&mut self) {
        self.state.select_first();
        self.search.on(self.songs_to_repr());
    }

    pub fn unordered_selected(&self) -> Option<usize> {
        self.state.selected().map(|i| self.search.real_i(i))
    }

    pub fn songs_to_repr(&self) -> Vec<String> {
        self.group
            .metadata
            .iter()
            .map(|m| {
                let repr = m.get("tracktitle");
                repr.map(|s| unidecode::unidecode(s)).unwrap_or_default()
            })
            .collect()
    }

    pub fn ordered_group(&self) -> Vec<(&HashMap<String, String>, &String)> {
        let search = &self.search;
        match search.state {
            SearchState::Off => self
                .group
                .metadata
                .iter()
                .zip(self.group.paths.iter())
                .collect(),
            _ => {
                let mut ordered = Vec::with_capacity(self.group.metadata.len());
                // clone not to hold up the guard
                let order = { search.result.read().unwrap().clone() };
                for i in order.iter() {
                    if let Some(m) = self.group.metadata.get(*i)
                        && let Some(path) = self.group.paths.get(*i)
                    {
                        ordered.push((m, path));
                    }
                }

                ordered
            }
        }
    }
}

impl Default for LibraryState {
    fn default() -> Self {
        Self {
            state: TableState::default(),
            focused_part: FocusedPart::default(),
            group_by_tags: vec!["albumartist".into(), "album".into()],
            children_tags: vec!["tracknumber".into(), "tracktitle".into()],
            children: Vec::new(),
            search: Search::default(),
        }
    }
}

impl Scroll for LibraryState {
    fn scroll(&mut self, delta: i32) {
        let u_delta = delta.unsigned_abs() as usize;
        let (n_rows, state) = match self.focused_part {
            FocusedPart::Groups => {
                if let Some(child) = self.selected_child_mut() {
                    child.search.off();
                }

                (self.children.len(), &mut self.state)
            }
            FocusedPart::Child(i) => (
                self.children[i].group.paths.len(),
                &mut self.children[i].state,
            ),
        };
        match state.selected() {
            Some(r) => {
                if delta < 0 {
                    if r >= u_delta {
                        state.scroll_up_by(u_delta as u16);
                    } else {
                        state.select(Some(n_rows - (u_delta - r)));
                    }
                } else {
                    if r + u_delta < n_rows {
                        state.scroll_down_by(u_delta as u16);
                    } else {
                        state.select(Some(u_delta - (n_rows - r)));
                    }
                }
            }
            None => state.select_first(),
        };
    }

    fn scroll_to_top(&mut self) {
        match self.focused_part {
            FocusedPart::Groups => self.state.select_first(),
            FocusedPart::Child(i) => self.children[i].state.select_first(),
        }
    }

    fn scroll_to_bottom(&mut self) {
        match self.focused_part {
            FocusedPart::Groups => self
                .state
                .select(Some(self.children.len().saturating_sub(1))),
            FocusedPart::Child(i) => {
                let n = self.children[i].group.paths.len();
                self.children[i].state.select(Some(n.saturating_sub(1)));
            }
        }
    }
}

impl LibraryState {
    pub fn search_on(&mut self) {
        self.scroll_to_top();
        self.search.on(self.children_to_repr());
    }

    pub fn unordered_selected(&self) -> Option<usize> {
        self.state.selected().map(|i| self.search.real_i(i))
    }

    pub fn children_to_repr(&self) -> Vec<String> {
        self.children
            .iter()
            .map(|child| {
                let mut repr = String::new();
                for value in &child.id_comb {
                    repr += value;
                }

                unidecode::unidecode(&repr)
            })
            .collect()
    }

    pub fn update(&mut self, grouped_songs: HashMap<Vec<String>, SongGroup>) {
        self.children.clear();
        for (id_comb, group) in grouped_songs {
            let child = LibraryChildState {
                state: TableState::default(),
                id_comb,
                group,
                search: Search::default(),
            };
            self.children.push(child);
        }
        self.children
            .sort_unstable_by(|lhs, rhs| (lhs.id_comb).cmp(&rhs.id_comb));
        if self.children.is_empty() {
            self.state.select(None);
        } else {
            self.state.select(Some(
                self.state
                    .selected()
                    .map_or(0, |i| i.min(self.children.len().saturating_sub(1))),
            ));
        }
        self.focus_left();
    }

    pub fn selected_child(&self) -> Option<&LibraryChildState> {
        self.unordered_selected().map(|i| &self.children[i])
    }

    pub fn selected_child_mut(&mut self) -> Option<&mut LibraryChildState> {
        self.unordered_selected().map(|i| &mut self.children[i])
    }

    pub fn selected_songs(&self) -> Option<&[String]> {
        match self.focused_part {
            FocusedPart::Groups => self
                .selected_child()
                .map(|child| &child.group.paths)
                .map(|v| &**v),
            FocusedPart::Child(i) => self.children[i]
                .unordered_selected()
                .map(|j| &self.children[i].group.paths[j..=j]),
        }
    }

    pub fn focus_left(&mut self) {
        if let Some(i) = self.state.selected() {
            self.state.select(Some(i));
            let real_i = self.search.real_i(i);
            self.children[real_i].state.select(None);
        }
        self.focused_part = FocusedPart::Groups;
    }

    pub fn focus_right(&mut self) {
        if let Some(i) = self.state.selected() {
            let real_i = self.search.real_i(i);
            self.children[real_i].state.select_first();
            self.focused_part = FocusedPart::Child(real_i);
        }
    }

    pub fn ordered_children(&self) -> Vec<&LibraryChildState> {
        let search = &self.search;
        match search.state {
            SearchState::Off => self.children.iter().collect(),
            _ => {
                let mut ordered = Vec::with_capacity(self.children.len());
                // clone not to hold up the guard
                let order = { search.result.read().unwrap().clone() };
                for child in order.iter().filter_map(|&i| self.children.get(i)) {
                    ordered.push(child);
                }

                ordered
            }
        }
    }
}
