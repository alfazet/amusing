pub trait Scroll {
    fn scroll(&mut self, delta: i32);
    fn scroll_to_top(&mut self);
    fn scroll_to_bottom(&mut self);
}
