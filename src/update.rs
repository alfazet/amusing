use anyhow::Result;
use ratatui::crossterm::event::{self, Event as TermEvent};

use crate::{
    app::{App, AppState, Screen},
    model::{
        common::Scroll,
        library::LibraryState,
        musing::{MusingState, MusingStateDelta},
    },
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
    Play,
    Seek(i64),
    Speed(i16),
    Volume(i8),
    MusingUpdate,
    Remove,
    Clear,
    QueueScroll(i32),
    QueueScrollToTop,
    QueueScrollToBottom,
    QueueStartSearch,
    LibraryScroll(i32),
    LibraryScrollToTop,
    LibraryScrollToBottom,
    LibraryFocusLeft,
    LibraryFocusRight,
    AddToQueue,
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

fn translate_key_event_queue(app: &App, ev: event::KeyEvent) -> Option<Message> {
    use event::KeyCode as Key;

    match ev.code {
        Key::Char('j') | Key::Down => Some(Message::Update(Update::QueueScroll(1))),
        Key::Char('k') | Key::Up => Some(Message::Update(Update::QueueScroll(-1))),
        Key::Home => Some(Message::Update(Update::QueueScrollToTop)),
        Key::End => Some(Message::Update(Update::QueueScrollToBottom)),
        Key::Char('d') => Some(Message::Update(Update::Remove)),
        Key::Delete => Some(Message::Update(Update::Clear)),
        Key::Enter => Some(Message::Update(Update::Play)),
        Key::Char('/') => Some(Message::Update(Update::QueueStartSearch)),
        _ => None,
    }
}

fn translate_key_event_library(app: &App, ev: event::KeyEvent) -> Option<Message> {
    use event::KeyCode as Key;

    match ev.code {
        Key::Char('j') | Key::Down => Some(Message::Update(Update::LibraryScroll(1))),
        Key::Char('k') | Key::Up => Some(Message::Update(Update::LibraryScroll(-1))),
        Key::Home => Some(Message::Update(Update::LibraryScrollToTop)),
        Key::End => Some(Message::Update(Update::LibraryScrollToBottom)),
        Key::Char('h') | Key::Left => Some(Message::Update(Update::LibraryFocusLeft)),
        Key::Char('l') | Key::Right => Some(Message::Update(Update::LibraryFocusRight)),
        Key::Enter => Some(Message::Update(Update::AddToQueue)),
        _ => None,
    }
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
        Key::Char('<') => Some(Message::Update(Update::Speed(-5))),
        Key::Char('>') => Some(Message::Update(Update::Speed(5))),
        Key::Char('-') => Some(Message::Update(Update::Volume(-5))),
        Key::Char('=') => Some(Message::Update(Update::Volume(5))),
        Key::Char('U') => Some(Message::Update(Update::MusingUpdate)),
        Key::Char('1') => Some(Message::SwitchScreen(Screen::Cover)),
        Key::Char('2') => Some(Message::SwitchScreen(Screen::Queue)),
        Key::Char('3') => Some(Message::SwitchScreen(Screen::Library)),
        _ => match app.screen {
            Screen::Queue => translate_key_event_queue(app, ev),
            Screen::Library => translate_key_event_library(app, ev),
            _ => None,
        },
    }
}

pub fn update_library(app: &mut App) -> Result<()> {
    app.connection
        .update()
        .map(|msg| app.status_msg = Some(msg))?;
    let grouped_songs = app.connection.grouped_songs(
        &app.library_state.group_by_tags,
        &app.library_state.children_tags,
    )?;
    app.library_state.update(grouped_songs);

    Ok(())
}

pub fn update_app(app: &mut App, msg: Message) {
    let _ = app.status_msg.take();
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
            Update::MusingUpdate => update_library(app),
            Update::QueueScroll(delta) => {
                app.queue_state.scroll(delta);
                Ok(())
            }
            Update::QueueScrollToTop => {
                app.queue_state.scroll_to_top();
                Ok(())
            }
            Update::QueueScrollToBottom => {
                app.queue_state.scroll_to_bottom();
                Ok(())
            }
            Update::QueueStartSearch => {
                app.queue_state.search_on();
                let _ = app
                    .queue_state
                    .search
                    .as_ref()
                    .unwrap()
                    .tx_pattern
                    .send(String::from("nevv"));
                Ok(())
            }
            Update::LibraryScroll(delta) => {
                app.library_state.scroll(delta);
                Ok(())
            }
            Update::LibraryScrollToTop => {
                app.library_state.scroll_to_top();
                Ok(())
            }
            Update::LibraryScrollToBottom => {
                app.library_state.scroll_to_bottom();
                Ok(())
            }
            Update::LibraryFocusLeft => {
                app.library_state.focus_left();
                Ok(())
            }
            Update::LibraryFocusRight => {
                app.library_state.focus_right();
                Ok(())
            }
            Update::AddToQueue => match app.library_state.selected_songs() {
                Some(songs) => app.connection.add_to_queue(songs),
                None => Ok(()),
            },
            Update::Play => match app.queue_state.unordered_selected() {
                Some(i) => app.connection.play(app.musing_state.queue[i].id),
                None => Ok(()),
            },
            Update::Remove => match app.queue_state.unordered_selected() {
                Some(i) => app.connection.remove(app.musing_state.queue[i].id),
                None => Ok(()),
            },
            Update::Seek(seconds) => app.connection.seek(seconds),
            Update::Speed(speed) => app.connection.speed(speed),
            Update::Volume(delta) => app.connection.volume(delta),
            other => app.connection.no_response(&enum_stringify!(other)),
        },
    };

    if let Err(e) = res {
        app.status_msg = Some(e.to_string());
    }
}

pub fn update_state(app: &mut App, delta: MusingStateDelta) {
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
    if let Some(current) = delta.current {
        app.musing_state.current = current;
    }
    if delta.timer.is_some() {
        app.musing_state.timer = delta.timer;
    }
}

pub fn update_metadata(app: &mut App) {
    let paths: Vec<_> = app
        .musing_state
        .queue
        .iter()
        .map(|song| song.path.as_str())
        .collect();
    match app.connection.metadata(&paths, None) {
        Ok(metadata) => app.queue_state.metadata = metadata,
        Err(e) => app.status_msg = Some(e.to_string()),
    }
}
