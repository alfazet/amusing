use anyhow::{Result, anyhow, bail};
use serde_json::{Map, Value};
use std::path::PathBuf;

#[derive(Debug, Default)]
pub enum PlaybackState {
    #[default]
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Default)]
pub enum PlaybackMode {
    #[default]
    Single,
    Sequential,
    Random,
}

#[derive(Debug, Default)]
pub struct MusingSong {
    id: u64,
    path: PathBuf,
}

#[derive(Debug, Default)]
pub struct MusingState {
    playback_state: PlaybackState,
    playback_mode: PlaybackMode,
    volume: u64,
    speed: u64,
    gapless: bool,
    devices: Vec<String>,
    queue: Vec<MusingSong>,
}

#[derive(Debug, Default)]
pub struct MusingStateDelta {
    playback_state: Option<PlaybackState>,
    playback_mode: Option<PlaybackMode>,
    volume: Option<u64>,
    speed: Option<u64>,
    gapless: Option<bool>,
    devices: Option<Vec<String>>,
    queue: Option<Vec<MusingSong>>,
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

                Ok(MusingSong {
                    id,
                    path: path.into(),
                })
            }
            None => bail!("expected a JSON object"),
        }
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

        Ok(Self {
            playback_state,
            playback_mode,
            volume,
            speed,
            gapless,
            devices,
            queue,
        })
    }
}
