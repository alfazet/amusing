use anyhow::{Result, anyhow};
use serde_json::{Map, Value};

#[derive(Debug)]
pub struct MusingState {
    volume: u8,
    // speed, current_song
}

// pub struct MusingQueue {
//     songs: Vec<Song>,
// }

impl TryFrom<&mut Map<String, Value>> for MusingState {
    type Error = anyhow::Error;

    fn try_from(args: &mut Map<String, Value>) -> Result<Self> {
        let volume: u8 = serde_json::from_value(
            args.remove("volume")
                .ok_or(anyhow!("key `volume` not found"))?,
        )?;

        Ok(Self { volume })
    }
}
