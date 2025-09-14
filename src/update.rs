use anyhow::Result;
use ratatui::crossterm::event::{self, Event as TermEvent};
use ratatui_image::thread::ResizeResponse;
use tui_input::backend::crossterm::EventHandler;

use crate::{
    app::{App, AppState, Screen},
    model::{
        common::{FocusedPart, Scroll},
        cover_art::CoverArtState,
        keybind::{Binding, KeybindNode},
        musing::MusingStateDelta,
        search::SearchState,
    },
};

#[derive(Debug)]
pub enum AppUpdate {
    Next,
    Previous,
    Pause,
    Resume,
    Toggle,
    Stop,
    Play,
    Seek(i64),
    Speed(i16),
    Volume(i8),
    Scroll(i32),
    ScrollTop,
    ScrollBottom,
    FocusLeft,
    FocusRight,
    StartSearch,
    EndSearch,
    IdleSearch,
    UpdateSearch,
    AddToQueue,
    RemoveFromQueue,
    ClearQueue,
    ModeGapless,
    ModeRandom,
    ModeSequential,
    ModeSingle,
    MusingUpdate,
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

fn translate_binding_queue(app: &mut App, binding: Binding) -> Option<Message> {
    let search = &mut app.queue_state.search;
    match search.state {
        SearchState::On => match binding {
            Binding::EndSearch => Some(Message::Update(AppUpdate::IdleSearch)),
            _ => {
                let term_ev = TermEvent::Key(*app.key_events.last().unwrap());
                search.input.handle_event(&term_ev);

                Some(Message::Update(AppUpdate::UpdateSearch))
            }
        },
        _ => match binding {
            Binding::ScrollUp => Some(Message::Update(AppUpdate::Scroll(-1))),
            Binding::ScrollDown => Some(Message::Update(AppUpdate::Scroll(1))),
            Binding::ScrollManyUp => Some(Message::Update(AppUpdate::Scroll(-5))),
            Binding::ScrollManyDown => Some(Message::Update(AppUpdate::Scroll(5))),
            Binding::ScrollTop => Some(Message::Update(AppUpdate::ScrollTop)),
            Binding::ScrollBottom => Some(Message::Update(AppUpdate::ScrollBottom)),
            Binding::RemoveFromQueue => Some(Message::Update(AppUpdate::RemoveFromQueue)),
            Binding::ClearQueue => Some(Message::Update(AppUpdate::ClearQueue)),
            Binding::Play => Some(Message::Update(AppUpdate::Play)),
            Binding::StartSearch => Some(Message::Update(AppUpdate::StartSearch)),
            Binding::EndSearch => Some(Message::Update(AppUpdate::EndSearch)),
            _ => translate_binding_common(app, binding),
        },
    }
}

fn translate_binding_library_both(app: &mut App, binding: Binding) -> Option<Message> {
    match binding {
        Binding::ScrollUp => Some(Message::Update(AppUpdate::Scroll(-1))),
        Binding::ScrollDown => Some(Message::Update(AppUpdate::Scroll(1))),
        Binding::ScrollManyUp => Some(Message::Update(AppUpdate::Scroll(-5))),
        Binding::ScrollManyDown => Some(Message::Update(AppUpdate::Scroll(5))),
        Binding::ScrollTop => Some(Message::Update(AppUpdate::ScrollTop)),
        Binding::ScrollBottom => Some(Message::Update(AppUpdate::ScrollBottom)),
        Binding::FocusLeft => Some(Message::Update(AppUpdate::FocusLeft)),
        Binding::FocusRight => Some(Message::Update(AppUpdate::FocusRight)),
        Binding::AddToQueue => Some(Message::Update(AppUpdate::AddToQueue)),
        Binding::StartSearch => Some(Message::Update(AppUpdate::StartSearch)),
        Binding::EndSearch => Some(Message::Update(AppUpdate::EndSearch)),
        _ => translate_binding_common(app, binding),
    }
}

fn translate_binding_library_groups(app: &mut App, binding: Binding) -> Option<Message> {
    let search = &mut app.library_state.search;
    match search.state {
        SearchState::On => match binding {
            Binding::EndSearch => Some(Message::Update(AppUpdate::IdleSearch)),
            _ => {
                let term_ev = TermEvent::Key(*app.key_events.last().unwrap());
                search.input.handle_event(&term_ev);

                Some(Message::Update(AppUpdate::UpdateSearch))
            }
        },
        _ => translate_binding_library_both(app, binding),
    }
}

fn translate_binding_library_child(app: &mut App, binding: Binding, i: usize) -> Option<Message> {
    let search = &mut app.library_state.children[i].search;
    match search.state {
        SearchState::On => match binding {
            Binding::EndSearch => Some(Message::Update(AppUpdate::IdleSearch)),
            _ => {
                let term_ev = TermEvent::Key(*app.key_events.last().unwrap());
                search.input.handle_event(&term_ev);

                Some(Message::Update(AppUpdate::UpdateSearch))
            }
        },
        _ => translate_binding_library_both(app, binding),
    }
}

pub fn translate_binding_common(app: &mut App, binding: Binding) -> Option<Message> {
    match binding {
        Binding::Quit => Some(Message::SwitchAppState(AppState::Done)),
        Binding::Next => Some(Message::Update(AppUpdate::Next)),
        Binding::Previous => Some(Message::Update(AppUpdate::Previous)),
        Binding::Pause => Some(Message::Update(AppUpdate::Pause)),
        Binding::Resume => Some(Message::Update(AppUpdate::Resume)),
        Binding::Toggle => Some(Message::Update(AppUpdate::Toggle)),
        Binding::Stop => Some(Message::Update(AppUpdate::Stop)),
        Binding::SeekForwards => Some(Message::Update(AppUpdate::Seek(app.config.seek_step))),
        Binding::SeekBackwards => Some(Message::Update(AppUpdate::Seek(-app.config.seek_step))),
        Binding::SpeedUp => Some(Message::Update(AppUpdate::Speed(app.config.speed_step))),
        Binding::SpeedDown => Some(Message::Update(AppUpdate::Speed(-app.config.speed_step))),
        Binding::VolumeUp => Some(Message::Update(AppUpdate::Volume(app.config.volume_step))),
        Binding::VolumeDown => Some(Message::Update(AppUpdate::Volume(-app.config.volume_step))),
        Binding::ModeSequential => Some(Message::Update(AppUpdate::ModeSequential)),
        Binding::ModeSingle => Some(Message::Update(AppUpdate::ModeSingle)),
        Binding::ModeRandom => Some(Message::Update(AppUpdate::ModeRandom)),
        Binding::ModeGapless => Some(Message::Update(AppUpdate::ModeGapless)),
        Binding::MusingUpdate => Some(Message::Update(AppUpdate::MusingUpdate)),
        Binding::ScreenCover => Some(Message::SwitchScreen(Screen::Cover)),
        Binding::ScreenQueue => Some(Message::SwitchScreen(Screen::Queue)),
        Binding::ScreenLibrary => Some(Message::SwitchScreen(Screen::Library)),
        _ => None,
    }
}

pub fn translate_key_event(app: &mut App, ev: event::KeyEvent) -> Option<Message> {
    app.key_events.push(ev);
    let default_translation = KeybindNode::Terminal(Binding::Other);
    let translation = app
        .config
        .keybind
        .translate(&app.key_events)
        .unwrap_or(&default_translation);
    match translation {
        KeybindNode::Terminal(binding) => {
            let res = match app.screen {
                Screen::Queue => translate_binding_queue(app, *binding),
                Screen::Library => match app.library_state.focused_part {
                    FocusedPart::Groups => translate_binding_library_groups(app, *binding),
                    FocusedPart::Child(i) => translate_binding_library_child(app, *binding, i),
                },
                _ => translate_binding_common(app, *binding),
            };
            app.key_events.clear();

            res
        }
        KeybindNode::Transition(_) => None,
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
            AppUpdate::ScrollTop => {
                match app.screen {
                    Screen::Queue => app.queue_state.scroll_to_top(),
                    Screen::Library => app.library_state.scroll_to_top(),
                    _ => (),
                }
                Ok(())
            }
            AppUpdate::ScrollBottom => {
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
                            app.library_state.children[i].search.idle();
                        }
                    },
                    _ => (),
                }
                Ok(())
            }
            AppUpdate::UpdateSearch => {
                match app.screen {
                    Screen::Queue => {
                        let pattern = app.queue_state.search.input.value().to_string();
                        app.queue_state.search.pattern_update(pattern);
                    }
                    Screen::Library => match app.library_state.focused_part {
                        FocusedPart::Groups => {
                            let pattern = app.library_state.search.input.value().to_string();
                            app.library_state.search.pattern_update(pattern);
                        }
                        FocusedPart::Child(i) => {
                            let child = &app.library_state.children[i];
                            let pattern = child.search.input.value().to_string();
                            app.library_state.children[i].search.pattern_update(pattern);
                        }
                    },
                    _ => (),
                }
                Ok(())
            }
            AppUpdate::FocusLeft => {
                if let Screen::Library = app.screen {
                    app.library_state.focus_left();
                }
                Ok(())
            }
            AppUpdate::FocusRight => {
                if let Screen::Library = app.screen {
                    app.library_state.focus_right();
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
            AppUpdate::RemoveFromQueue => match app.queue_state.unordered_selected() {
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

pub fn update_state(app: &mut App, delta: MusingStateDelta, cover_art_state: &mut CoverArtState) {
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
    if let Some(current) = delta.current {
        app.musing_state.current = current;
    }
    if let Some(cover_art) = delta.cover_art {
        let _ = cover_art_state.replace_art(cover_art.as_ref());
        app.musing_state.cover_art = cover_art;
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

pub fn update_cover_art(cover_art: &mut CoverArtState, resize: ResizeResponse) {
    let _ = cover_art.state.update_resized_protocol(resize);
}
