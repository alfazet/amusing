use anyhow::{Result, anyhow, bail};
use serde_json::{Map, Value};
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

#[derive(Clone, Copy, Debug, Default)]
pub enum PlaybackState {
    #[default]
    Stopped,
    Playing,
    Paused,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum PlaybackMode {
    #[default]
    Single,
    Sequential,
    Random,
}

#[derive(Clone, Debug, Default)]
pub struct MusingSong {
    pub id: u64,
    pub path: String,
}

#[derive(Debug, Default)]
pub struct MusingState {
    pub playback_state: PlaybackState,
    pub playback_mode: PlaybackMode,
    pub volume: u64,
    pub speed: u64,
    pub gapless: bool,
    pub devices: Vec<String>,
    pub queue: Vec<MusingSong>,
    pub current: Option<u64>,
    pub timer: Option<(u64, u64)>,
}

#[derive(Debug, Default)]
pub struct MusingStateDelta {
    pub playback_state: Option<PlaybackState>,
    pub playback_mode: Option<PlaybackMode>,
    pub volume: Option<u64>,
    pub speed: Option<u64>,
    pub gapless: Option<bool>,
    pub devices: Option<Vec<String>>,
    pub queue: Option<Vec<MusingSong>>,
    pub current: Option<u64>,
    pub timer: Option<(u64, u64)>,
}

impl Display for PlaybackState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            PlaybackState::Stopped => "stopped",
            PlaybackState::Playing => "playing",
            PlaybackState::Paused => "paused",
        }
        .to_string();

        write!(f, "{}", s)
    }
}

impl TryFrom<&Value> for PlaybackState {
    type Error = anyhow::Error;

    fn try_from(value: &Value) -> Result<Self> {
        match value.as_str() {
            Some("stopped") => Ok(PlaybackState::Stopped),
            Some("playing") => Ok(PlaybackState::Playing),
            Some("paused") => Ok(PlaybackState::Paused),
            Some(other) => bail!("unexpected playback state `{}`", other),
            None => bail!("expected a string"),
        }
    }
}

impl Display for PlaybackMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            PlaybackMode::Sequential => "W e r",
            PlaybackMode::Single => "w E r",
            PlaybackMode::Random => "w e R",
        }
        .to_string();

        write!(f, "{}", s)
    }
}

impl TryFrom<&Value> for PlaybackMode {
    type Error = anyhow::Error;

    fn try_from(value: &Value) -> Result<Self> {
        match value.as_str() {
            Some("sequential") => Ok(PlaybackMode::Sequential),
            Some("single") => Ok(PlaybackMode::Single),
            Some("random") => Ok(PlaybackMode::Random),
            Some(other) => bail!("unexpected playback mode `{}`", other),
            None => bail!("expected a string"),
        }
    }
}

impl TryFrom<&Value> for MusingSong {
    type Error = anyhow::Error;

    fn try_from(value: &Value) -> Result<Self> {
        match value.as_object() {
            Some(object) => {
                let id = object
                    .get("id")
                    .and_then(|x| x.as_u64())
                    .ok_or(anyhow!("expected key `id`"))?;
                let path = object
                    .get("path")
                    .and_then(|x| x.as_str().map(|s| s.to_string()))
                    .ok_or(anyhow!("expected key `path`"))?;

                Ok(MusingSong { id, path })
            }
            None => bail!("expected a JSON object"),
        }
    }
}

impl MusingState {
    pub fn is_stopped(&self) -> bool {
        matches!(self.playback_state, PlaybackState::Stopped)
    }
}

impl TryFrom<Value> for MusingStateDelta {
    type Error = anyhow::Error;

    fn try_from(mut value: Value) -> Result<Self> {
        let object = value
            .as_object_mut()
            .ok_or(anyhow!("expected a JSON object"))?;
        let playback_state = object
            .remove("playback_state")
            .as_ref()
            .and_then(|x| PlaybackState::try_from(x).ok());
        let playback_mode = object
            .remove("playback_mode")
            .as_ref()
            .and_then(|x| PlaybackMode::try_from(x).ok());
        let volume = object.remove("volume").and_then(|x| x.as_u64());
        let speed = object.remove("speed").and_then(|x| x.as_u64());
        let gapless = object.remove("gapless").and_then(|x| x.as_bool());
        let devices = object
            .remove("devices")
            .and_then(|x| {
                x.as_array().map(|v| {
                    v.iter()
                        .map(|s| s.as_str().map(|s| s.to_string()))
                        .collect::<Option<_>>()
                })
            })
            .flatten();
        let queue = object.remove("queue").and_then(|x| {
            x.as_array().and_then(|v| {
                v.iter()
                    .map(|s| MusingSong::try_from(s).ok())
                    .collect::<Option<_>>()
            })
        });
        let current = object.remove("current").and_then(|x| x.as_u64());
        let timer = object.remove("timer").map(|timer| match timer.as_object() {
            Some(timer) => {
                let elapsed = timer
                    .get("elapsed")
                    .and_then(|x| x.as_u64())
                    .unwrap_or_default();
                let duration = timer
                    .get("duration")
                    .and_then(|x| x.as_u64())
                    .unwrap_or_default();

                (elapsed, duration)
            }
            None => (0, 0),
        });

        Ok(Self {
            playback_state,
            playback_mode,
            volume,
            speed,
            gapless,
            devices,
            queue,
            current,
            timer,
        })
    }
}
