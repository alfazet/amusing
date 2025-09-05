use anyhow::{Result, anyhow, bail};
use serde_json::{Map, Value, json};
use std::{
    collections::HashMap,
    io::{BufReader, prelude::*},
    net::{Shutdown, TcpStream},
};

use crate::model::musing::{MusingState, MusingStateDelta};

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
        if let Some(obj) = value.as_object()
            && let Some(status) = obj.get("status")
            && status == "err"
            && let Some(reason) = obj.get("reason")
        {
            bail!("musing error: {}", reason.to_string());
        }

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

    pub fn metadata(
        &mut self,
        paths: &[&str],
        tags: Option<&[&str]>,
    ) -> Result<Vec<HashMap<String, String>>> {
        let mut request = Map::new();
        request.insert("kind".into(), "metadata".into());
        request.insert("paths".into(), paths.into());
        let _ = match tags {
            Some(tags) => request.insert("tags".into(), tags.into()),
            None => request.insert("all_tags".into(), true.into()),
        };
        self.write_msg(Value::from(request))?;
        let mut res = self.read_msg()?;
        if let Some(obj) = res.as_object_mut()
            && let Some(mut metadata) = obj.remove("metadata")
            && let Some(metadata) = metadata.as_array_mut()
        {
            let metadata: Vec<_> = metadata
                .iter()
                .filter_map(|v| v.as_object())
                .map(|obj| {
                    obj.iter()
                        .filter_map(|(k, v)| {
                            v.is_string()
                                .then(|| (k.clone(), v.as_str().unwrap().to_string()))
                        })
                        .collect::<HashMap<_, _>>()
                })
                .collect();

            Ok(metadata)
        } else {
            bail!("could not fetch metadata");
        }
    }

    pub fn all_songs(&mut self, group_by: &[&str]) -> Result<HashMap<Vec<String>, Vec<String>>> {
        // returns a map like:
        // { [some combo of groupby tags]: [list of tracktitles] }
        // then whenever we want to add some tracks to the queue we query for their paths with
        // an appropriate select
        // this makes it so that a full metadata fetch happens only for songs that we add to the
        // queue
        bail!("abc");
    }

    pub fn state_delta(&mut self) -> Result<MusingStateDelta> {
        let request = json!({
            "kind": "state",
        });
        self.write_msg(request)?;
        let res = self.read_msg()?;

        MusingStateDelta::try_from(res)
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

    pub fn play(&mut self, id: u64) -> Result<()> {
        let request = json!({
            "kind": "play",
            "id": id,
        });
        self.write_msg(request)?;

        self.read_msg().map(|_| ())
    }

    pub fn remove(&mut self, id: u64) -> Result<()> {
        let request = json!({
            "kind": "removequeue",
            "ids": [id],
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
