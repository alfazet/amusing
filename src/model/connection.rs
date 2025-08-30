use anyhow::{Result, anyhow};
use serde_json::{Value, json};
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
    pub fn try_new(port: u16) -> Result<Self> {
        let stream = TcpStream::connect(format!("127.0.0.1:{}", port))?;
        // TODO: refactor reading/writing to not repeat it everywhere
        let mut stream = BufReader::new(stream);
        let mut version_len_bytes: [u8; 4] = [0; 4];
        stream.read_exact(&mut version_len_bytes)?;
        let version_len = u32::from_be_bytes(version_len_bytes) as usize;
        let mut version_bytes = vec![0; version_len];
        stream.read_exact(&mut version_bytes);
        let version = String::from_utf8(version_bytes)?;

        Ok(Self { version, stream })
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.stream
            .get_ref()
            .shutdown(Shutdown::Both)
            .map_err(|e| e.into())
    }

    pub fn state(&mut self) -> Result<MusingState> {
        let req_str = json!({
            "kind": "state",
        })
        .to_string();
        let req_bytes = req_str.as_bytes();
        let req_len_bytes = (req_bytes.len() as u32).to_be_bytes();
        self.stream.get_mut().write_all(&req_len_bytes)?;
        self.stream.get_mut().write_all(req_bytes)?;

        let mut resp_len_bytes: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut resp_len_bytes)?;
        let resp_len = u32::from_be_bytes(resp_len_bytes) as usize;
        let mut resp_bytes = vec![0; resp_len];
        self.stream.read_exact(&mut resp_bytes)?;
        let resp = String::from_utf8(resp_bytes)?;
        let mut resp_json = serde_json::from_str::<Value>(&resp)?;
        let resp_object = resp_json
            .as_object_mut()
            .ok_or(anyhow!("a response must be a JSON object"))?;

        MusingState::try_from(resp_object)
    }
}
