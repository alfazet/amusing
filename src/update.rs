use anyhow::Result;
use ratatui::crossterm::event::{self, Event as TermEvent};
use tui_input::backend::crossterm::EventHandler;

use crate::{
    app::{App, AppState, Screen},
    model::{
        common::{FocusedPart, Scroll},
        library::LibraryState,
        musing::{MusingState, MusingStateDelta},
        search::SearchState,
    },
};

#[derive(Debug)]
pub enum AppUpdate {
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
    SearchUpdate(String),
    AddToQueue,
}

#[derive(Debug)]
pub enum Message {
    SwitchScreen(Screen),
    SwitchAppState(AppState),
    Update(AppUpdate),
}

macro_rules! enum_stringify {
    ($variant:expr) => {{
        let s = format!("{:?}", $variant);
        s.split("::").last().unwrap().to_lowercase()
    }};
}

fn translate_key_event_queue(app: &mut App, ev: event::KeyEvent) -> Option<Message> {
    use event::KeyCode as Key;

    let search = &mut app.queue_state.search;
    match search.state {
        SearchState::On => match ev.code {
            Key::Enter | Key::Esc => Some(Message::Update(AppUpdate::IdleSearch)),
            _ => {
                let term_ev = TermEvent::Key(ev);
                search.input.handle_event(&term_ev);

                Some(Message::Update(AppUpdate::SearchUpdate(
                    search.input.value().to_string(),
                )))
            }
        },
        _ => match ev.code {
            // TODO: <C-d>/<C-u>
            Key::Char('j') | Key::Down => Some(Message::Update(AppUpdate::Scroll(1))),
            Key::Char('k') | Key::Up => Some(Message::Update(AppUpdate::Scroll(-1))),
            Key::Home => Some(Message::Update(AppUpdate::ScrollToTop)),
            Key::End => Some(Message::Update(AppUpdate::ScrollToBottom)),
            Key::Char('d') => Some(Message::Update(AppUpdate::Remove)),
            Key::Delete => Some(Message::Update(AppUpdate::Clear)),
            Key::Enter => Some(Message::Update(AppUpdate::Play)),
            Key::Char('/') => Some(Message::Update(AppUpdate::StartSearch)),
            Key::Esc => Some(Message::Update(AppUpdate::EndSearch)),
            _ => translate_key_event_common(app, ev),
        },
    }
}

fn translate_key_event_library_both(app: &mut App, ev: event::KeyEvent) -> Option<Message> {
    use event::KeyCode as Key;

    match ev.code {
        Key::Char('j') | Key::Down => Some(Message::Update(AppUpdate::Scroll(1))),
        Key::Char('k') | Key::Up => Some(Message::Update(AppUpdate::Scroll(-1))),
        Key::Home => Some(Message::Update(AppUpdate::ScrollToTop)),
        Key::End => Some(Message::Update(AppUpdate::ScrollToBottom)),
        Key::Char('h') | Key::Left => Some(Message::Update(AppUpdate::FocusLeft)),
        Key::Char('l') | Key::Right => Some(Message::Update(AppUpdate::FocusRight)),
        Key::Enter => Some(Message::Update(AppUpdate::AddToQueue)),
        Key::Char('/') => Some(Message::Update(AppUpdate::StartSearch)),
        Key::Esc => Some(Message::Update(AppUpdate::EndSearch)),
        _ => translate_key_event_common(app, ev),
    }
}

fn translate_key_event_library_groups(app: &mut App, ev: event::KeyEvent) -> Option<Message> {
    use event::KeyCode as Key;

    let search = &mut app.library_state.search;
    match search.state {
        SearchState::On => match ev.code {
            Key::Enter | Key::Esc => Some(Message::Update(AppUpdate::IdleSearch)),
            _ => {
                let term_ev = TermEvent::Key(ev);
                search.input.handle_event(&term_ev);

                Some(Message::Update(AppUpdate::SearchUpdate(
                    search.input.value().to_string(),
                )))
            }
        },
        _ => translate_key_event_library_both(app, ev),
    }
}

fn translate_key_event_library_child(
    app: &mut App,
    ev: event::KeyEvent,
    i: usize,
) -> Option<Message> {
    use event::KeyCode as Key;

    let search = &mut app.library_state.children[i].search;
    match search.state {
        SearchState::On => match ev.code {
            Key::Enter | Key::Esc => Some(Message::Update(AppUpdate::IdleSearch)),
            _ => {
                let term_ev = TermEvent::Key(ev);
                search.input.handle_event(&term_ev);

                Some(Message::Update(AppUpdate::SearchUpdate(
                    search.input.value().to_string(),
                )))
            }
        },
        _ => translate_key_event_library_both(app, ev),
    }
}

pub fn translate_key_event_common(app: &mut App, ev: event::KeyEvent) -> Option<Message> {
    use event::KeyCode as Key;

    match ev.code {
        Key::Char('q') => Some(Message::SwitchAppState(AppState::Done)),
        Key::Char('w') => Some(Message::Update(AppUpdate::Sequential)),
        Key::Char('e') => Some(Message::Update(AppUpdate::Single)),
        Key::Char('r') => Some(Message::Update(AppUpdate::Random)),
        Key::Char('P') => Some(Message::Update(AppUpdate::Previous)),
        Key::Char('N') => Some(Message::Update(AppUpdate::Next)),
        Key::Char('g') => Some(Message::Update(AppUpdate::Gapless)),
        Key::Char(' ') => Some(Message::Update(AppUpdate::Toggle)),
        Key::Char('p') => Some(Message::Update(AppUpdate::Pause)),
        Key::Char('o') => Some(Message::Update(AppUpdate::Resume)),
        Key::Char('S') => Some(Message::Update(AppUpdate::Stop)),
        Key::Char('[') => Some(Message::Update(AppUpdate::Seek(-5))),
        Key::Char(']') => Some(Message::Update(AppUpdate::Seek(5))),
        Key::Char('<') => Some(Message::Update(AppUpdate::Speed(-5))),
        Key::Char('>') => Some(Message::Update(AppUpdate::Speed(5))),
        Key::Char('-') => Some(Message::Update(AppUpdate::Volume(-5))),
        Key::Char('=') => Some(Message::Update(AppUpdate::Volume(5))),
        Key::Char('U') => Some(Message::Update(AppUpdate::MusingUpdate)),
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
        Screen::Library => match app.library_state.focused_part {
            FocusedPart::Groups => translate_key_event_library_groups(app, ev),
            FocusedPart::Child(i) => {
                let i = app.library_state.search.real_i(i);
                translate_key_event_library_child(app, ev, i)
            }
        },
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
            AppUpdate::MusingUpdate => update_library(app),
            AppUpdate::Scroll(delta) => {
                match app.screen {
                    Screen::Queue => app.queue_state.scroll(delta),
                    Screen::Library => app.library_state.scroll(delta),
                    _ => (),
                }
                Ok(())
            }
            AppUpdate::ScrollToTop => {
                match app.screen {
                    Screen::Queue => app.queue_state.scroll_to_top(),
                    Screen::Library => app.library_state.scroll_to_top(),
                    _ => (),
                }
                Ok(())
            }
            AppUpdate::ScrollToBottom => {
                match app.screen {
                    Screen::Queue => app.queue_state.scroll_to_bottom(),
                    Screen::Library => app.library_state.scroll_to_bottom(),
                    _ => (),
                }
                Ok(())
            }
            AppUpdate::StartSearch => {
                match app.screen {
                    Screen::Queue => app.queue_state.search_on(),
                    Screen::Library => match app.library_state.focused_part {
                        FocusedPart::Groups => app.library_state.search_on(),
                        FocusedPart::Child(i) => {
                            let i = app.library_state.search.real_i(i);
                            app.library_state.children[i].search_on();
                        }
                    },
                    _ => (),
                }
                Ok(())
            }
            AppUpdate::EndSearch => {
                match app.screen {
                    Screen::Queue => app.queue_state.search.off(),
                    Screen::Library => match app.library_state.focused_part {
                        FocusedPart::Groups => app.library_state.search.off(),
                        FocusedPart::Child(i) => {
                            let i = app.library_state.search.real_i(i);
                            app.library_state.children[i].search.off();
                        }
                    },
                    _ => (),
                }
                Ok(())
            }
            AppUpdate::IdleSearch => {
                match app.screen {
                    Screen::Queue => app.queue_state.search.idle(),
                    Screen::Library => match app.library_state.focused_part {
                        FocusedPart::Groups => app.library_state.search.idle(),
                        FocusedPart::Child(i) => {
                            let i = app.library_state.search.real_i(i);
                            app.library_state.children[i].search.idle();
                        }
                    },
                    _ => (),
                }
                Ok(())
            }
            AppUpdate::SearchUpdate(pattern) => {
                match app.screen {
                    Screen::Queue => app.queue_state.search.pattern_update(pattern),
                    Screen::Library => match app.library_state.focused_part {
                        FocusedPart::Groups => app.library_state.search.pattern_update(pattern),
                        FocusedPart::Child(i) => {
                            let i = app.library_state.search.real_i(i);
                            app.library_state.children[i].search.pattern_update(pattern);
                        }
                    },
                    _ => (),
                }
                Ok(())
            }
            AppUpdate::FocusLeft => {
                match app.screen {
                    Screen::Library => app.library_state.focus_left(),
                    Screen::Playlists => (),
                    _ => (),
                }
                Ok(())
            }
            AppUpdate::FocusRight => {
                match app.screen {
                    Screen::Library => app.library_state.focus_right(),
                    Screen::Playlists => (),
                    _ => (),
                }
                Ok(())
            }
            AppUpdate::AddToQueue => match app.library_state.selected_songs() {
                Some(songs) => app.connection.add_to_queue(songs),
                None => Ok(()),
            },
            AppUpdate::Play => match app.queue_state.unordered_selected() {
                Some(i) => app.connection.play(app.musing_state.queue[i].id),
                None => Ok(()),
            },
            AppUpdate::Remove => match app.queue_state.unordered_selected() {
                Some(i) => app.connection.remove(app.musing_state.queue[i].id),
                None => Ok(()),
            },
            AppUpdate::Seek(seconds) => app.connection.seek(seconds),
            AppUpdate::Speed(speed) => app.connection.speed(speed),
            AppUpdate::Volume(delta) => app.connection.volume(delta),
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
                .search
                .list_update(app.queue_state.metadata_to_repr());
        }
        Err(e) => app.status_msg = Some(e.to_string()),
    }
}
