use fuzzy_matcher::skim::SkimMatcherV2;
use rayon::prelude::*;
use std::{
    sync::{Arc, RwLock, mpsc as std_chan},
    thread,
};
use tui_input::Input as TuiInput;

#[derive(Debug)]
pub enum SearchMessage {
    NewPattern(String),
    NewList(Vec<String>),
}

#[derive(Debug)]
pub struct SearchState {
    pub tx: std_chan::Sender<SearchMessage>,
    pub result: Arc<RwLock<Vec<usize>>>,
    pub input: TuiInput,
    pub active: bool,
}

fn compute_ordering(matcher: &SkimMatcherV2, list: &[String], pattern: &str) -> Vec<usize> {
    let mut scores: Vec<_> = list
        .par_iter()
        .enumerate()
        // negative sign because we want to sort descending
        .map(|(i, s)| (-matcher.fuzzy(s, pattern, false).unwrap_or_default().0, i))
        .collect();
    scores.par_sort_unstable();

    scores.into_iter().map(|(_, i)| i).collect()
}

pub fn run(
    mut list: Vec<String>,
    rx: std_chan::Receiver<SearchMessage>,
    result: Arc<RwLock<Vec<usize>>>,
) {
    thread::spawn(move || {
        let matcher = SkimMatcherV2::default().ignore_case();
        let mut pattern = String::new();
        while let Ok(msg) = rx.recv() {
            match msg {
                SearchMessage::NewPattern(new_pattern) => {
                    pattern = unidecode::unidecode(&new_pattern);
                    *result.write().unwrap() = compute_ordering(&matcher, &list, &pattern);
                }
                SearchMessage::NewList(new_list) => {
                    list = new_list;
                    *result.write().unwrap() = compute_ordering(&matcher, &list, &pattern);
                }
            }
        }
    });
}
