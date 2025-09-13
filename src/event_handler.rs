use anyhow::Result;
use ratatui::crossterm::event::{self, Event as TermEvent};
use ratatui_image::{errors::Errors, thread::ResizeResponse};
use std::{
    sync::mpsc as std_chan,
    thread,
    time::{Duration, Instant},
};

use crate::panic;

// in ms
const REFRESH_TIMEOUT: u64 = 125;
const POLL_TIMEOUT: u64 = 16;

pub enum Event {
    Keypress(event::KeyEvent),
    CoverArtResize(Result<ResizeResponse, Errors>),
    Refresh,
}

pub fn run(tx_event: std_chan::Sender<Event>) {
    let refresh_timer = Duration::from_millis(REFRESH_TIMEOUT);
    let poll_timer = Duration::from_millis(POLL_TIMEOUT);
    thread::spawn(move || {
        panic::register_backtrace_panic_handler();
        let _ = tx_event.send(Event::Refresh);
        let mut last_refresh = Instant::now();
        loop {
            if event::poll(poll_timer).expect("event poll failed")
                && let TermEvent::Key(ev) = event::read().expect("event read failed")
                && let event::KeyEventKind::Press = ev.kind
            {
                let _ = tx_event.send(Event::Keypress(ev));
            }
            let now = Instant::now();
            if now - last_refresh >= refresh_timer {
                let _ = tx_event.send(Event::Refresh);
                last_refresh = now;
            }
        }
    });
}
