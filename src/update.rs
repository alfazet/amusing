use anyhow::Result;
use ratatui::crossterm::event::{self, Event as TermEvent};

use crate::{
    app::{App, AppState, Screen},
    model::musing::{MusingState, MusingStateDelta},
};

#[derive(Debug)]
pub enum Update {
    Gapless,
    Random,
    Sequential,
    Single,
    Previous,
    Next,
    Pause,
    Resume,
    Toggle,
    Stop,
    Seek(i64),
    Speed(i16),
    Volume(i8),
}

#[derive(Debug)]
pub enum Message {
    SwitchScreen(Screen),
    SwitchAppState(AppState),
    Update(Update),
}

macro_rules! enum_stringify {
    ($variant:expr) => {{
        let s = format!("{:?}", $variant);
        s.split("::").last().unwrap().to_lowercase()
    }};
}

pub fn translate_key_event(app: &App, ev: event::KeyEvent) -> Option<Message> {
    use event::KeyCode as Key;

    // TODO: make bindings configurable (a map (Message,  KeyEvent))
    match ev.code {
        Key::Char('q') => Some(Message::SwitchAppState(AppState::Done)),
        Key::Char('w') => Some(Message::Update(Update::Sequential)),
        Key::Char('e') => Some(Message::Update(Update::Single)),
        Key::Char('r') => Some(Message::Update(Update::Random)),
        Key::Char('P') => Some(Message::Update(Update::Previous)),
        Key::Char('N') => Some(Message::Update(Update::Next)),
        Key::Char('g') => Some(Message::Update(Update::Gapless)),
        Key::Char(' ') => Some(Message::Update(Update::Toggle)),
        Key::Char('p') => Some(Message::Update(Update::Pause)),
        Key::Char('o') => Some(Message::Update(Update::Resume)),
        Key::Char('S') => Some(Message::Update(Update::Stop)),
        Key::Char('[') => Some(Message::Update(Update::Seek(-5))),
        Key::Char(']') => Some(Message::Update(Update::Seek(5))),
        Key::Char(',') => Some(Message::Update(Update::Speed(-5))),
        Key::Char('.') => Some(Message::Update(Update::Speed(5))),
        Key::Char('-') => Some(Message::Update(Update::Volume(-5))),
        Key::Char('=') => Some(Message::Update(Update::Volume(5))),
        _ => None,
    }
}

pub fn update_app(app: &mut App, msg: Message) {
    let res = match msg {
        Message::SwitchScreen(screen) => {
            app.screen = screen;
            Ok(())
        }
        Message::SwitchAppState(app_state) => {
            app.app_state = app_state;
            Ok(())
        }
        Message::Update(update) => match update {
            Update::Seek(seconds) => app.connection.seek(seconds),
            Update::Speed(speed) => app.connection.speed(speed),
            Update::Volume(delta) => app.connection.volume(delta),
            other => app.connection.no_response(&enum_stringify!(other)),
        },
    };

    // if let Err(e) = res {
    //     app.status_msg = Some(e.to_string());
    // }
}

pub fn main_update(app: &mut App, delta: MusingStateDelta) {
    if let Some(playback_state) = delta.playback_state {
        app.musing_state.playback_state = playback_state;
    }
    if let Some(playback_mode) = delta.playback_mode {
        app.musing_state.playback_mode = playback_mode;
    }
    if let Some(volume) = delta.volume {
        app.musing_state.volume = volume;
    }
    if let Some(speed) = delta.speed {
        app.musing_state.speed = speed;
    }
    if let Some(gapless) = delta.gapless {
        app.musing_state.gapless = gapless;
    }
    if let Some(devices) = delta.devices {
        app.musing_state.devices = devices;
    }
    if let Some(queue) = delta.queue {
        app.musing_state.queue = queue;
        update_metadata(app);
    }
    if delta.current.is_some() {
        app.musing_state.current = delta.current;
    }
}

pub fn update_metadata(app: &mut App) {
    let paths: Vec<_> = app
        .musing_state
        .queue
        .iter()
        .map(|song| song.path.as_str())
        .collect();
    if let Ok(metadata) = app.connection.metadata(&paths, None) {
        app.metadata = metadata;
    }
}
