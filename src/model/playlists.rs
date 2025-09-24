use ratatui::widgets::TableState;
use std::collections::HashMap;

use crate::model::{
    common::{FocusedPart, Scroll, SongGroup},
    search::{Search, SearchState},
};

#[derive(Debug, Default)]
pub struct PlaylistsChildState {
    pub state: TableState,
    pub path: String,
    pub group: SongGroup,
    pub search: Search,
}

#[derive(Default, Debug)]
pub struct PlaylistsState {
    pub state: TableState,
    pub focused_part: FocusedPart,
    pub children_tags: Vec<String>,
    pub children: Vec<PlaylistsChildState>, // playlists
    pub search: Search,
}

impl PlaylistsChildState {
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
            .zip(self.group.paths.iter())
            .map(|(m, path)| {
                let repr = m.get("tracktitle").unwrap_or(path);
                unidecode::unidecode(repr)
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

impl Scroll for PlaylistsState {
    fn scroll(&mut self, delta: i32) {
        if self.children.is_empty() {
            return;
        }
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
                } else if r + u_delta < n_rows {
                    state.scroll_down_by(u_delta as u16);
                } else {
                    state.select(Some(u_delta - (n_rows - r)));
                }
            }
            None => state.select_first(),
        };
    }

    fn scroll_to_top(&mut self) {
        if self.children.is_empty() {
            return;
        }
        match self.focused_part {
            FocusedPart::Groups => self.state.select_first(),
            FocusedPart::Child(i) => self.children[i].state.select_first(),
        }
    }

    fn scroll_to_bottom(&mut self) {
        if self.children.is_empty() {
            return;
        }
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

impl PlaylistsState {
    pub fn new(children_tags: Vec<String>) -> Self {
        Self {
            state: TableState::default(),
            focused_part: FocusedPart::default(),
            children_tags,
            children: Vec::new(),
            search: Search::default(),
        }
    }

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
            .map(|child| unidecode::unidecode(&child.path))
            .collect()
    }

    pub fn update(&mut self, playlists: HashMap<String, SongGroup>) {
        self.children.clear();
        for (path, group) in playlists {
            let child = PlaylistsChildState {
                state: TableState::default(),
                path,
                group,
                search: Search::default(),
            };
            self.children.push(child);
        }
        self.children
            .sort_unstable_by(|lhs, rhs| (lhs.path).cmp(&rhs.path));
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

    pub fn selected_child(&self) -> Option<&PlaylistsChildState> {
        self.unordered_selected().map(|i| &self.children[i])
    }

    pub fn selected_child_mut(&mut self) -> Option<&mut PlaylistsChildState> {
        self.unordered_selected().map(|i| &mut self.children[i])
    }

    pub fn selected_songs(&self) -> Option<&[String]> {
        match self.focused_part {
            FocusedPart::Groups => self
                .selected_child()
                .map(|child| &child.group.paths)
                .map(|v| &**v),
            FocusedPart::Child(i) => self.children.get(i).and_then(|child| {
                child
                    .unordered_selected()
                    .map(|j| &self.children[i].group.paths[j..=j])
            }),
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

    pub fn ordered_children(&self) -> Vec<&PlaylistsChildState> {
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
