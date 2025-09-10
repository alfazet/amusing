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

#[derive(Debug, Default)]
pub enum SearchState {
    #[default]
    Off,
    On,
    Idle,
}

#[derive(Debug, Default)]
pub struct Search {
    pub tx: Option<std_chan::Sender<SearchMessage>>,
    pub result: Arc<RwLock<Vec<usize>>>,
    pub input: TuiInput,
    pub state: SearchState,
}

impl Search {
    pub fn on(&mut self, list: Vec<String>) {
        // ensure that the old search thread ends
        self.off();
        let (tx, rx) = std_chan::channel();
        let result = Arc::new(RwLock::new((0..list.len()).collect()));
        self.tx = Some(tx);
        self.result = Arc::clone(&result);
        self.state = SearchState::On;
        run(list, rx, result);
    }

    pub fn off(&mut self) {
        let _ = self.tx.take();
        self.input = TuiInput::default();
        self.state = SearchState::Off;
    }

    pub fn idle(&mut self) {
        self.state = SearchState::Idle;
    }

    pub fn pattern_update(&self, pattern: String) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(SearchMessage::NewPattern(pattern));
        }
    }

    pub fn list_update(&self, list: Vec<String>) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(SearchMessage::NewList(list));
        }
    }

    pub fn real_i(&self, i: usize) -> usize {
        match &self.tx {
            Some(_) => {
                let order = self.result.read().unwrap();
                (*order).get(i).copied().unwrap_or_default()
            }
            None => i,
        }
    }
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
