use anyhow::{Result, anyhow};
use serde_json::{Map, Value, json};
use std::{
    io::{BufReader, prelude::*},
    net::{Shutdown, TcpStream},
};

use crate::model::musing::MusingState;

#[derive(Debug)]
pub struct Connection {
    version: String,
    stream: BufReader<TcpStream>,
}

impl Connection {
    fn read_msg(&mut self) -> Result<Value> {
        let mut len_bytes: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut len_bytes)?;
        let len = u32::from_be_bytes(len_bytes) as usize;
        let mut bytes = vec![0; len];
        self.stream.read_exact(&mut bytes)?;
        let resp = String::from_utf8(bytes)?;
        let value = serde_json::from_str::<Value>(&resp)?;

        Ok(value)
    }

    fn write_msg(&mut self, msg: Value) -> Result<()> {
        let msg = msg.to_string();
        let bytes = msg.as_bytes();
        let len_bytes = (bytes.len() as u32).to_be_bytes();
        self.stream.get_mut().write_all(&len_bytes)?;
        self.stream.get_mut().write_all(bytes)?;

        Ok(())
    }

    pub fn try_new(port: u16) -> Result<Self> {
        let stream = TcpStream::connect(format!("127.0.0.1:{}", port))?;
        let mut stream = BufReader::new(stream);
        let mut version_len_bytes: [u8; 4] = [0; 4];
        stream.read_exact(&mut version_len_bytes)?;
        let version_len = u32::from_be_bytes(version_len_bytes) as usize;
        let mut version_bytes = vec![0; version_len];
        stream.read_exact(&mut version_bytes)?;
        let version = String::from_utf8(version_bytes)?;

        Ok(Self { version, stream })
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.stream
            .get_ref()
            .shutdown(Shutdown::Both)
            .map_err(|e| e.into())
    }

    pub fn metadata(&mut self, paths: Vec<String>, tags: Option<Vec<String>>) -> Result<Value> {
        let mut request = Map::new();
        request.insert("paths".into(), paths.into());
        let _ = match tags {
            Some(tags) => request.insert("tags".into(), tags.into()),
            None => request.insert("all_tags".into(), true.into()),
        };
        self.write_msg(Value::from(request))?;

        self.read_msg()
    }

    pub fn add_metadata(&mut self, delta: &mut Value) -> Result<()> {
        if let Some(queue) = delta.as_object_mut().and_then(|obj| obj.get_mut("queue"))
            && let Some(songs) = queue.as_array_mut()
        {
            for song in songs.iter_mut() {
                if let Some(song_obj) = song.as_object_mut()
                    && let Some(path) = song_obj.get("path").and_then(|s| s.as_str())
                {
                    song_obj.insert("metadata".into(), self.metadata(vec![path.into()], None)?);
                }
            }
        }

        Ok(())
    }

    pub fn state_delta(&mut self) -> Result<Value> {
        let request = json!({
            "kind": "state",
        });
        self.write_msg(request)?;

        // TODO: react to err key being present
        self.read_msg()
    }

    pub fn seek(&mut self, seconds: i64) -> Result<()> {
        let request = json!({
            "kind": "seek",
            "seconds": seconds,
        });
        self.write_msg(request)?;

        self.read_msg().map(|_| ())
    }

    pub fn speed(&mut self, delta: i16) -> Result<()> {
        let request = json!({
            "kind": "speed",
            "delta": delta,
        });
        self.write_msg(request)?;

        self.read_msg().map(|_| ())
    }

    pub fn volume(&mut self, delta: i8) -> Result<()> {
        let request = json!({
            "kind": "volume",
            "delta": delta,
        });
        self.write_msg(request)?;

        self.read_msg().map(|_| ())
    }

    // a convenience function for sending requests that don't have neither
    // any additional arguments nor a meaningful positive response
    pub fn no_response(&mut self, kind: &str) -> Result<()> {
        let request = json!({
            "kind": kind,
        });
        self.write_msg(request)?;

        self.read_msg().map(|_| ())
    }
}
