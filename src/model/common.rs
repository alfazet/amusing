pub trait Scroll {
    fn scroll(&mut self, delta: i32);
    fn scroll_to_top(&mut self);
    fn scroll_to_bottom(&mut self);
}

pub trait Search {
    fn order_by_score(&mut self, pattern: &str);
}

pub struct SearchState {
    pub pattern: Option<String>,
    pub active: bool,
}
