use anyhow::Result;
use ratatui::widgets::{ListState, TableState};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, mpsc as std_chan},
};
use tui_input::Input as TuiInput;

use crate::model::{
    common::{Scroll, Search},
    search::{self, SearchMessage, SearchState},
};

#[derive(Debug, Default)]
pub enum FocusedPart {
    #[default]
    Groups, // "lhs" of the view
    Child(usize), // "rhs" of the view, which child is focused
}

#[derive(Debug, Default)]
pub struct SongGroup {
    pub metadata: Vec<HashMap<String, String>>,
    pub paths: Vec<String>,
}

#[derive(Debug, Default)]
pub struct LibraryChildState {
    pub state: TableState,
    pub id_comb: Vec<String>, // the combination of tags that identifies these songs
    pub group: SongGroup,
}

#[derive(Debug)]
pub struct LibraryState {
    pub state: TableState,
    pub focused_part: FocusedPart,
    pub group_by_tags: Vec<String>,
    pub children_tags: Vec<String>,
    pub children: Vec<LibraryChildState>, // grouped collections of songs
    pub search: Option<SearchState>,
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

// impl Search for LibraryChildState {}

impl Default for LibraryState {
    fn default() -> Self {
        Self {
            state: TableState::default(),
            focused_part: FocusedPart::default(),
            group_by_tags: vec!["albumartist".into(), "album".into()],
            children_tags: vec!["tracknumber".into(), "tracktitle".into()],
            children: Vec::new(),
            search: None,
        }
    }
}

impl Scroll for LibraryState {
    fn scroll(&mut self, delta: i32) {
        let u_delta = delta.unsigned_abs() as usize;
        let (n_rows, state) = match self.focused_part {
            FocusedPart::Groups => (self.children.len(), &mut self.state),
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

impl Search for LibraryState {
    fn search_on(&mut self) {
        let (tx, rx) = std_chan::channel();
        let result = Arc::new(RwLock::new((0..self.children.len()).collect()));
        let list = self.children_to_repr();
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

impl LibraryState {
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

    // translates the "view" i into the "actual" i
    fn real_i(&self, i: usize) -> usize {
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
    fn unordered_selected(&self) -> Option<usize> {
        self.state.selected().map(|i| self.real_i(i))
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
                .state
                .selected()
                .map(|j| &self.children[i].group.paths[j..=j]),
        }
    }

    pub fn focus_left(&mut self) {
        if let Some(i) = self.state.selected() {
            self.state.select(Some(i));
            let real_i = self.real_i(i);
            self.children[real_i].state.select(None);
        }
        self.focused_part = FocusedPart::Groups;
    }

    pub fn focus_right(&mut self) {
        if let Some(i) = self.state.selected() {
            let real_i = self.real_i(i);
            self.children[real_i].state.select_first();
            self.focused_part = FocusedPart::Child(real_i);
        }
    }

    pub fn ordered_children(&self) -> Vec<&LibraryChildState> {
        match &self.search {
            Some(search) => {
                let mut ordered = Vec::with_capacity(self.children.len());
                // clone not to hold up the guard
                let order = { search.result.read().unwrap().clone() };
                for child in order.iter().filter_map(|&i| self.children.get(i)) {
                    ordered.push(child);
                }
                // let max_i = order.into_iter().max().unwrap_or_default();
                // add any items that weren't there back when we started the search
                // if max_i < self.children.len().saturating_sub(1) {
                //     for m in &self.metadata[(max_i + 1)..] {
                //         ordered.push(m);
                //     }
                // }

                ordered
            }
            None => self.children.iter().collect(),
        }
    }
}
