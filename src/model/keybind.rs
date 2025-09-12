use anyhow::{Result, anyhow, bail};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{collections::HashMap, str::FromStr, string::ParseError};
use strum_macros::EnumString;
use toml::{Table, Value as TomlValue};

// TODO: context aware bindings
#[derive(Clone, Copy, Debug, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Binding {
    Quit,
    Next,
    Previous,
    Pause,
    Resume,
    Toggle,
    Stop,
    Play,
    SeekForwards,
    SeekBackwards,
    SpeedUp,
    SpeedDown,
    VolumeUp,
    VolumeDown,
    ScrollUp,
    ScrollDown,
    ScrollManyUp,
    ScrollManyDown,
    ScrollTop,
    ScrollBottom,
    FocusLeft,
    FocusRight,
    StartSearch,
    EndSearch,
    AddToQueue,
    RemoveFromQueue,
    ClearQueue,
    ModeGapless,
    ModeRandom,
    ModeSequential,
    ModeSingle,
    MusingUpdate,
    ScreenCover,
    ScreenQueue,
    ScreenLibrary,
    // used to pass typed characters to search
    Other,
}

#[derive(Debug)]
pub enum KeybindNode {
    Transition(Keybind), // this key combinations leads to some binding(s)
    Terminal(Binding),   // this key combination corresponds to a binding
}

#[derive(Debug)]
pub struct Keybind(HashMap<KeyEvent, KeybindNode>);

impl Default for Keybind {
    fn default() -> Self {
        use KeyModifiers as Mods;
        use KeybindNode as Node;

        let mut keybind = Keybind(HashMap::new());
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('q'), Mods::NONE)],
            Binding::Quit,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('N'), Mods::NONE)],
            Binding::Next,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('P'), Mods::NONE)],
            Binding::Previous,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('p'), Mods::NONE)],
            Binding::Pause,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('o'), Mods::NONE)],
            Binding::Resume,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char(' '), Mods::NONE)],
            Binding::Toggle,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('S'), Mods::NONE)],
            Binding::Stop,
        );
        keybind.add_keybind(&[KeyEvent::new(KeyCode::Enter, Mods::NONE)], Binding::Play);
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char(']'), Mods::NONE)],
            Binding::SeekForwards,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('['), Mods::NONE)],
            Binding::SeekBackwards,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('>'), Mods::NONE)],
            Binding::SpeedUp,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('<'), Mods::NONE)],
            Binding::SpeedDown,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('='), Mods::NONE)],
            Binding::VolumeUp,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('-'), Mods::NONE)],
            Binding::VolumeDown,
        );
        keybind.add_keybind(&[KeyEvent::new(KeyCode::Up, Mods::NONE)], Binding::ScrollUp);
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('k'), Mods::NONE)],
            Binding::ScrollUp,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Down, Mods::NONE)],
            Binding::ScrollDown,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('j'), Mods::NONE)],
            Binding::ScrollDown,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('u'), Mods::CONTROL)],
            Binding::ScrollManyUp,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('d'), Mods::CONTROL)],
            Binding::ScrollManyDown,
        );
        keybind.add_keybind(
            &[
                KeyEvent::new(KeyCode::Char('g'), Mods::NONE),
                KeyEvent::new(KeyCode::Char('g'), Mods::NONE),
            ],
            Binding::ScrollTop,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('G'), Mods::NONE)],
            Binding::ScrollBottom,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('h'), Mods::NONE)],
            Binding::FocusLeft,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Left, Mods::NONE)],
            Binding::FocusLeft,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('l'), Mods::NONE)],
            Binding::FocusRight,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Right, Mods::NONE)],
            Binding::FocusRight,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('/'), Mods::NONE)],
            Binding::StartSearch,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Esc, Mods::NONE)],
            Binding::EndSearch,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('a'), Mods::NONE)],
            Binding::AddToQueue,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('d'), Mods::NONE)],
            Binding::RemoveFromQueue,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Delete, Mods::NONE)],
            Binding::ClearQueue,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('t'), Mods::NONE)],
            Binding::ModeGapless,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('r'), Mods::NONE)],
            Binding::ModeRandom,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('e'), Mods::NONE)],
            Binding::ModeSingle,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('w'), Mods::NONE)],
            Binding::ModeSequential,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('U'), Mods::NONE)],
            Binding::MusingUpdate,
        );

        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('1'), Mods::NONE)],
            Binding::ScreenCover,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('2'), Mods::NONE)],
            Binding::ScreenQueue,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('3'), Mods::NONE)],
            Binding::ScreenLibrary,
        );

        keybind
    }
}

impl TryFrom<Table> for Keybind {
    type Error = anyhow::Error;

    fn try_from(table: Table) -> Result<Self> {
        let mut keybind = Keybind::default();
        for (key, val) in table {
            let binding = Binding::from_str(&key)?;
            if let Some(s) = val.as_str() {
                let key_events = try_into_key_events(s)
                    .ok_or(anyhow!("could not parse `{}` as a keybinding", s))?;
                keybind.add_keybind(&key_events, binding);
            } else if let Some(a) = val.as_array() {
                for s in a.into_iter().filter_map(|s| s.as_str()) {
                    let key_events = try_into_key_events(s)
                        .ok_or(anyhow!("could not parse `{}` as a keybinding", s))?;
                    keybind.add_keybind(&key_events, binding);
                }
            } else {
                bail!("expected a string or an array");
            }
        }

        Ok(keybind)
    }
}

impl Keybind {
    pub fn add_keybind(&mut self, events: &[KeyEvent], binding: Binding) {
        match events.len() {
            0 => (),
            1 => {
                // only one key event, so this is a terminal
                self.0.insert(events[0], KeybindNode::Terminal(binding));
            }
            _ => {
                match self.0.get_mut(&events[0]) {
                    Some(node) => match node {
                        // overwrite with a new transition node
                        KeybindNode::Terminal(_) => {
                            self.0.insert(
                                events[0],
                                KeybindNode::Transition(Keybind(HashMap::new())),
                            );
                            self.add_keybind(&events, binding);
                        }
                        // add to the transition node
                        KeybindNode::Transition(trans) => trans.add_keybind(&events[1..], binding),
                    },
                    None => {
                        // add a new transition node
                        self.0
                            .insert(events[0], KeybindNode::Transition(Keybind(HashMap::new())));
                        self.add_keybind(&events, binding);
                    }
                }
            }
        }
    }

    // Some(Terminal) => this sequence matched something
    // Some(Transition) => this sequence will potentially match something
    // None => this sequence doesn't match and won't match anything in the future
    pub fn translate(&self, events: &[KeyEvent]) -> Option<&KeybindNode> {
        match events.len() {
            0 => None,
            1 => self.0.get(&events[0]),
            _ => match self.0.get(&events[0]) {
                Some(KeybindNode::Transition(trans)) => trans.translate(&events[1..]),
                other => other,
            },
        }
    }
}

fn try_into_code(s: impl AsRef<str>) -> Option<KeyCode> {
    let s = s.as_ref();
    if s.len() == 1 {
        s.chars().next().map(KeyCode::Char)
    } else {
        match s {
            "<SPACE>" => Some(KeyCode::Char(' ')),
            "<UP_ARROW>" => Some(KeyCode::Up),
            "<DOWN_ARROW>" => Some(KeyCode::Down),
            "<LEFT_ARROW>" => Some(KeyCode::Left),
            "<RIGHT_ARROW>" => Some(KeyCode::Right),
            "<ENTER>" => Some(KeyCode::Enter),
            "<ESCAPE>" => Some(KeyCode::Esc),
            "<TAB>" => Some(KeyCode::Tab),
            "<BACKSPACE>" => Some(KeyCode::Backspace),
            "<DELETE>" => Some(KeyCode::Delete),
            "<HOME>" => Some(KeyCode::Home),
            "<END>" => Some(KeyCode::End),
            "<PAGE_UP>" => Some(KeyCode::PageUp),
            "<PAGE_DOWN>" => Some(KeyCode::PageDown),
            _ => None,
        }
    }
}

fn try_into_key_events(s: impl AsRef<str>) -> Option<Vec<KeyEvent>> {
    let mut res = Vec::new();
    for part in s.as_ref().split_whitespace() {
        if let Some(code) = part.strip_prefix("C-").and_then(try_into_code) {
            res.push(KeyEvent::new(code, KeyModifiers::CONTROL));
        } else if let Some(code) = part.strip_prefix("S-").and_then(try_into_code) {
            res.push(KeyEvent::new(code, KeyModifiers::SHIFT));
        } else if let Some(code) = part.strip_prefix("C-S-").and_then(try_into_code) {
            res.push(KeyEvent::new(
                code,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            ));
        } else if let Some(code) = try_into_code(part) {
            res.push(KeyEvent::new(code, KeyModifiers::NONE));
        } else {
            return None;
        }
    }

    Some(res)
}
