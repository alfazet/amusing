use anyhow::Result;
use ratatui::crossterm::event::{self, Event as TermEvent};
use tui_input::backend::crossterm::EventHandler;

use crate::{
    app::{App, AppState, Screen},
    model::{
        common::{Scroll, Search},
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
    Scroll(i32),
    ScrollToTop,
    ScrollToBottom,
    FocusLeft,
    FocusRight,
    StartSearch,
    EndSearch,
    IdleSearch,
    UpdateSearch(String),
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

/*
fn translate_key_event_search(app: &mut App, ev: event::KeyEvent) -> Option<Message> {
    match ev.code {
        Key::Enter | Key::Esc => Some(Message::Update(Update::IdleSearch)),
        _ => {
            let term_ev = TermEvent::Key(ev);
            search.input.handle_event(&term_ev);

            Some(Message::Update(Update::UpdateSearch(
                search.input.value().to_string(),
            )))
        }
    }
}
*/

fn translate_key_event_queue(app: &mut App, ev: event::KeyEvent) -> Option<Message> {
    use event::KeyCode as Key;

    match &mut app.queue_state.search {
        Some(search) if search.active => match ev.code {
            Key::Enter | Key::Esc => Some(Message::Update(Update::IdleSearch)),
            _ => {
                let term_ev = TermEvent::Key(ev);
                search.input.handle_event(&term_ev);

                Some(Message::Update(Update::UpdateSearch(
                    search.input.value().to_string(),
                )))
            }
        },
        _ => match ev.code {
            // TODO: <C-d>/<C-u>
            Key::Char('j') | Key::Down => Some(Message::Update(Update::Scroll(1))),
            Key::Char('k') | Key::Up => Some(Message::Update(Update::Scroll(-1))),
            Key::Home => Some(Message::Update(Update::ScrollToTop)),
            Key::End => Some(Message::Update(Update::ScrollToBottom)),
            Key::Char('d') => Some(Message::Update(Update::Remove)),
            Key::Delete => Some(Message::Update(Update::Clear)),
            Key::Enter => Some(Message::Update(Update::Play)),
            Key::Char('/') => Some(Message::Update(Update::StartSearch)),
            Key::Esc => Some(Message::Update(Update::EndSearch)),
            _ => translate_key_event_common(app, ev),
        },
    }
}

fn translate_key_event_library(app: &mut App, ev: event::KeyEvent) -> Option<Message> {
    use event::KeyCode as Key;

    match &mut app.library_state.search {
        Some(search) if search.active => match ev.code {
            Key::Enter | Key::Esc => Some(Message::Update(Update::IdleSearch)),
            _ => {
                let term_ev = TermEvent::Key(ev);
                search.input.handle_event(&term_ev);

                Some(Message::Update(Update::UpdateSearch(
                    search.input.value().to_string(),
                )))
            }
        },
        _ => match ev.code {
            Key::Char('j') | Key::Down => Some(Message::Update(Update::Scroll(1))),
            Key::Char('k') | Key::Up => Some(Message::Update(Update::Scroll(-1))),
            Key::Home => Some(Message::Update(Update::ScrollToTop)),
            Key::End => Some(Message::Update(Update::ScrollToBottom)),
            Key::Char('h') | Key::Left => Some(Message::Update(Update::FocusLeft)),
            Key::Char('l') | Key::Right => Some(Message::Update(Update::FocusRight)),
            Key::Enter => Some(Message::Update(Update::AddToQueue)),
            Key::Char('/') => Some(Message::Update(Update::StartSearch)),
            Key::Esc => Some(Message::Update(Update::EndSearch)),
            _ => translate_key_event_common(app, ev),
        },
    }
}

pub fn translate_key_event_common(app: &mut App, ev: event::KeyEvent) -> Option<Message> {
    use event::KeyCode as Key;

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
        _ => None,
    }
}

pub fn translate_key_event(app: &mut App, ev: event::KeyEvent) -> Option<Message> {
    // TODO: make bindings configurable (a map (Message,  KeyEvent))
    match app.screen {
        Screen::Queue => translate_key_event_queue(app, ev),
        Screen::Library => translate_key_event_library(app, ev),
        _ => translate_key_event_common(app, ev),
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
            Update::Scroll(delta) => {
                match app.screen {
                    Screen::Queue => app.queue_state.scroll(delta),
                    Screen::Library => app.library_state.scroll(delta),
                    _ => (),
                }
                Ok(())
            }
            Update::ScrollToTop => {
                match app.screen {
                    Screen::Queue => app.queue_state.scroll_to_top(),
                    Screen::Library => app.library_state.scroll_to_top(),
                    _ => (),
                }
                Ok(())
            }
            Update::ScrollToBottom => {
                match app.screen {
                    Screen::Queue => app.queue_state.scroll_to_bottom(),
                    Screen::Library => app.library_state.scroll_to_bottom(),
                    _ => (),
                }
                Ok(())
            }
            Update::StartSearch => {
                match app.screen {
                    Screen::Queue => {
                        app.queue_state.scroll_to_top();
                        app.queue_state.search_on();
                    }
                    Screen::Library => {
                        app.library_state.scroll_to_top();
                        app.library_state.search_on();
                    }
                    _ => (),
                }
                Ok(())
            }
            Update::EndSearch => {
                match app.screen {
                    Screen::Queue => app.queue_state.search_off(),
                    Screen::Library => app.library_state.search_off(),
                    _ => (),
                }
                Ok(())
            }
            Update::IdleSearch => {
                match app.screen {
                    Screen::Queue => app.queue_state.search_idle(),
                    Screen::Library => app.library_state.search_idle(),
                    _ => (),
                }
                Ok(())
            }
            Update::UpdateSearch(pattern) => {
                match app.screen {
                    Screen::Queue => app.queue_state.search_pattern_update(pattern),
                    Screen::Library => app.library_state.search_pattern_update(pattern),
                    _ => (),
                }
                Ok(())
            }
            Update::FocusLeft => {
                match app.screen {
                    Screen::Library => app.library_state.focus_left(),
                    Screen::Playlists => (),
                    _ => (),
                }
                Ok(())
            }
            Update::FocusRight => {
                match app.screen {
                    Screen::Library => app.library_state.focus_right(),
                    Screen::Playlists => (),
                    _ => (),
                }
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
    if let Some(playlists) = delta.playlists {
        app.musing_state.playlists = playlists;
    }
    if let Some(devices) = delta.devices {
        app.musing_state.devices = devices;
    }
    if let Some(current) = delta.current {
        app.musing_state.current = current;
    }
    if delta.timer.is_some() {
        app.musing_state.timer = delta.timer;
    }
    if let Some(queue) = delta.queue {
        app.musing_state.queue = queue;
        update_queue(app);
    }
}

pub fn update_queue(app: &mut App) {
    let paths: Vec<_> = app
        .musing_state
        .queue
        .iter()
        .map(|song| song.path.as_str())
        .collect();
    match app.connection.metadata(&paths, None) {
        Ok(metadata) => {
            app.queue_state.metadata = metadata;
            app.queue_state
                .search_list_update(app.queue_state.metadata_to_repr());
        }
        Err(e) => app.status_msg = Some(e.to_string()),
    }
}
