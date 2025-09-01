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

    pub fn state_delta(&mut self) -> Result<Value> {
        let request = json!({
            "kind": "state",
        });
        self.write_msg(request)?;

        self.read_msg()
    }
}
