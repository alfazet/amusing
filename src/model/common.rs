use std::collections::HashMap;

pub trait Scroll {
    fn scroll(&mut self, delta: i32);
    fn scroll_to_top(&mut self);
    fn scroll_to_bottom(&mut self);
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
