use anyhow::{Result, anyhow, bail};
use serde_json::Value;
use std::fmt::{self, Display, Formatter};

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
    pub queue: Vec<MusingSong>,
    pub current: Option<u64>,
    pub cover_art: Option<String>, // base64 encoded
    pub timer: Option<(u64, u64)>,
}

#[derive(Debug, Default)]
pub struct MusingStateDelta {
    pub playback_state: Option<PlaybackState>,
    pub playback_mode: Option<PlaybackMode>,
    pub volume: Option<u64>,
    pub speed: Option<u64>,
    pub gapless: Option<bool>,
    pub queue: Option<Vec<MusingSong>>,
    pub current: Option<Option<u64>>,
    pub cover_art: Option<Option<String>>,
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

impl From<&Value> for PlaybackState {
    fn from(value: &Value) -> Self {
        match value.as_str() {
            Some("stopped") => PlaybackState::Stopped,
            Some("playing") => PlaybackState::Playing,
            Some("paused") => PlaybackState::Paused,
            _ => unreachable!(),
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

impl From<&Value> for PlaybackMode {
    fn from(value: &Value) -> Self {
        match value.as_str() {
            Some("sequential") => PlaybackMode::Sequential,
            Some("single") => PlaybackMode::Single,
            Some("random") => PlaybackMode::Random,
            _ => unreachable!(),
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
            .map(PlaybackState::from);
        let playback_mode = object
            .remove("playback_mode")
            .as_ref()
            .map(PlaybackMode::from);
        let volume = object.remove("volume").and_then(|x| x.as_u64());
        let speed = object.remove("speed").and_then(|x| x.as_u64());
        let gapless = object.remove("gapless").and_then(|x| x.as_bool());
        let queue = object.remove("queue").and_then(|x| {
            x.as_array().and_then(|v| {
                v.iter()
                    .map(|s| MusingSong::try_from(s).ok())
                    .collect::<Option<_>>()
            })
        });
        let current = object
            .remove("current")
            .map(|x| if x.is_null() { None } else { x.as_u64() });
        let cover_art = object.remove("cover_art").map(|x| {
            if x.is_null() {
                None
            } else {
                x.as_str().map(|s| s.to_string())
            }
        });
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
            queue,
            current,
            cover_art,
            timer,
        })
    }
}
