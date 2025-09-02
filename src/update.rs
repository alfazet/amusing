use anyhow::Result;

use crate::model::musing::{MusingState, MusingStateDelta};

pub fn update_musing_state(state: &mut MusingState, delta: MusingStateDelta) {
    if let Some(playback_state) = delta.playback_state {
        state.playback_state = playback_state;
    }
    if let Some(playback_mode) = delta.playback_mode {
        state.playback_mode = playback_mode;
    }
    if let Some(volume) = delta.volume {
        state.volume = volume;
    }
    if let Some(speed) = delta.speed {
        state.speed = speed;
    }
    if let Some(gapless) = delta.gapless {
        state.gapless = gapless;
    }
    if let Some(devices) = delta.devices {
        state.devices = devices;
    }
    if let Some(queue) = delta.queue {
        state.queue = queue;
    }
}
