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
}
