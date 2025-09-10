use std::collections::HashMap;

pub trait Scroll {
    fn scroll(&mut self, delta: i32);
    fn scroll_to_top(&mut self);
    fn scroll_to_bottom(&mut self);
}

pub trait Search {
    fn search_on(&mut self);
    fn search_off(&mut self);
    fn search_idle(&mut self);
    fn search_pattern_update(&mut self, pattern: String);
    fn search_list_update(&mut self, list: Vec<String>);
    // translates the "view" i into the "actual" i
    fn real_i(&self, i: usize) -> usize;
    // returns the currently selected index, but taking into account
    // the fact that the view may be sorted in a different order
    fn unordered_selected(&self) -> Option<usize>;
}

// for use in screens where the view is split into two parts,
#[derive(Debug, Default)]
pub enum FocusedPart {
    #[default]
    Groups, // "lhs" of the view
    Child(usize), // "rhs" of the view, which child is focused
}

// for grouping songs (represents an album or a playlist)
#[derive(Debug, Default)]
pub struct SongGroup {
    pub metadata: Vec<HashMap<String, String>>,
    pub paths: Vec<String>,
}
