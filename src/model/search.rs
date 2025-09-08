use fuzzy_matcher::skim::SkimMatcherV2;
use rayon::prelude::*;
use std::{
    sync::{Arc, RwLock, mpsc as std_chan},
    thread,
};
use tui_input::Input as TuiInput;

#[derive(Debug)]
pub struct SearchState {
    pub tx_pattern: std_chan::Sender<Option<String>>,
    pub result: Arc<RwLock<Vec<usize>>>,
    pub input: TuiInput,
    pub active: bool,
}

pub fn run(
    list: Vec<String>,
    rx_pattern: std_chan::Receiver<Option<String>>,
    result: Arc<RwLock<Vec<usize>>>,
) {
    thread::spawn(move || {
        let matcher = SkimMatcherV2::default().ignore_case();
        while let Ok(pattern) = rx_pattern.recv()
            && let Some(pattern) = pattern
        {
            let mut scores: Vec<_> = list
                .par_iter()
                .enumerate()
                // negative sign because we want to sort descending
                .map(|(i, s)| (-matcher.fuzzy(s, &pattern, false).unwrap_or_default().0, i))
                .collect();
            scores.par_sort_unstable();
            *result.write().unwrap() = scores.into_iter().map(|(_, i)| i).collect();
        }
    });
}
