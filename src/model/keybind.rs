use anyhow::{Result, anyhow, bail};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{collections::HashMap, str::FromStr, string::ParseError};
use strum_macros::EnumString;
use toml::{Table, Value as TomlValue};

#[derive(Clone, Copy, Debug, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Binding {
    Previous,
    Next,
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
            &[KeyEvent::new(KeyCode::Down, Mods::NONE)],
            Binding::ScrollDown,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('j'), Mods::NONE)],
            Binding::ScrollDown,
        );
        keybind.add_keybind(&[KeyEvent::new(KeyCode::Up, Mods::NONE)], Binding::ScrollUp);
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('k'), Mods::NONE)],
            Binding::ScrollUp,
        );
        keybind.add_keybind(
            &[KeyEvent::new(KeyCode::Char('G'), Mods::NONE)],
            Binding::ScrollBottom,
        );
        keybind.add_keybind(
            &[
                KeyEvent::new(KeyCode::Char('g'), Mods::NONE),
                KeyEvent::new(KeyCode::Char('g'), Mods::NONE),
            ],
            Binding::ScrollTop,
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

    pub fn translate(&self, events: &[KeyEvent]) -> Option<Binding> {
        match events.len() {
            0 => None,
            1 => match self.0.get(&events[0]) {
                Some(KeybindNode::Terminal(binding)) => Some(*binding),
                _ => None,
            },
            _ => match self.0.get(&events[0]) {
                Some(KeybindNode::Transition(trans)) => trans.translate(&events[1..]),
                _ => None,
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
