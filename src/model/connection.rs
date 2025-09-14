use anyhow::{Result, bail};
use serde_json::{Map, Value as JsonValue, json};
use std::{
    collections::HashMap,
    io::{BufReader, prelude::*},
    net::TcpStream,
    sync::mpsc as std_chan,
    thread,
};

use crate::{
    constants,
    event_handler::Event,
    model::{common::SongGroup, musing::MusingStateDelta},
};

#[derive(Debug)]
pub enum MusingRequest {
    Metadata(Vec<String>, Option<Vec<String>>),
    GroupedSongs(Vec<String>, Vec<String>),
    StateDelta,
    Seek(i64),
    Speed(i16),
    Volume(i8),
    AddToQueue(Vec<String>),
    Play(u64),
    Remove(u64),
    Update,
    Other(String),
}

#[derive(Debug)]
pub enum MusingResponse {
    Error(String),
    Metadata(Vec<HashMap<String, String>>),
    GroupedSongs(HashMap<Vec<String>, SongGroup>),
    StateDelta(MusingStateDelta),
    Update(String),
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Connection {
    version: String,
    tx: std_chan::Sender<MusingRequest>,
}

impl Connection {
    pub fn try_new(port: u16, tx_response: std_chan::Sender<Event>) -> Result<Self> {
        let stream = TcpStream::connect(format!("127.0.0.1:{}", port))?;
        let mut stream = BufReader::new(stream);
        let mut version_len_bytes: [u8; 4] = [0; 4];
        stream.read_exact(&mut version_len_bytes)?;
        let version_len = u32::from_be_bytes(version_len_bytes) as usize;
        let mut version_bytes = vec![0; version_len];
        stream.read_exact(&mut version_bytes)?;
        let version = String::from_utf8(version_bytes)?;

        let (tx_request, rx_request) = std_chan::channel();
        thread::spawn(move || run(stream, tx_response, rx_request));

        Ok(Self {
            version,
            tx: tx_request,
        })
    }

    pub fn send(&self, request: MusingRequest) {
        let _ = self.tx.send(request);
    }
}

fn run(
    mut stream: BufReader<TcpStream>,
    tx: std_chan::Sender<Event>,
    rx: std_chan::Receiver<MusingRequest>,
) {
    use Event::MusingResponse as Resp;

    while let Ok(request) = rx.recv() {
        let _ = match request {
            MusingRequest::Metadata(paths, tags) => match metadata(&mut stream, paths, tags) {
                Ok(metadata) => tx.send(Resp(MusingResponse::Metadata(metadata))),
                Err(e) => tx.send(Resp(MusingResponse::Error(e.to_string()))),
            },
            MusingRequest::GroupedSongs(group_by, children_tags) => {
                match grouped_songs(&mut stream, group_by, children_tags) {
                    Ok(grouped) => tx.send(Resp(MusingResponse::GroupedSongs(grouped))),
                    Err(e) => tx.send(Resp(MusingResponse::Error(e.to_string()))),
                }
            }
            MusingRequest::StateDelta => match state_delta(&mut stream) {
                Ok(delta) => tx.send(Resp(MusingResponse::StateDelta(delta))),
                Err(e) => tx.send(Resp(MusingResponse::Error(e.to_string()))),
            },
            MusingRequest::Seek(seconds) => {
                let res = seek(&mut stream, seconds);
                res.map(|_| ())
                    .or_else(|e| tx.send(Resp(MusingResponse::Error(e.to_string()))))
            }
            MusingRequest::Speed(delta) => {
                let res = speed(&mut stream, delta);
                res.map(|_| ())
                    .or_else(|e| tx.send(Resp(MusingResponse::Error(e.to_string()))))
            }
            MusingRequest::Volume(delta) => {
                let res = volume(&mut stream, delta);
                res.map(|_| ())
                    .or_else(|e| tx.send(Resp(MusingResponse::Error(e.to_string()))))
            }
            MusingRequest::AddToQueue(paths) => {
                let res = add_to_queue(&mut stream, paths);
                res.map(|_| ())
                    .or_else(|e| tx.send(Resp(MusingResponse::Error(e.to_string()))))
            }
            MusingRequest::Play(id) => {
                let res = play(&mut stream, id);
                res.map(|_| ())
                    .or_else(|e| tx.send(Resp(MusingResponse::Error(e.to_string()))))
            }
            MusingRequest::Remove(id) => {
                let res = remove(&mut stream, id);
                res.map(|_| ())
                    .or_else(|e| tx.send(Resp(MusingResponse::Error(e.to_string()))))
            }
            MusingRequest::Update => match update(&mut stream) {
                Ok(res) => tx.send(Resp(MusingResponse::Update(res))),
                Err(e) => tx.send(Resp(MusingResponse::Error(e.to_string()))),
            },
            MusingRequest::Other(endpoint) => {
                let res = other(&mut stream, endpoint);
                res.map(|_| ())
                    .or_else(|e| tx.send(Resp(MusingResponse::Error(e.to_string()))))
            }
        };
    }
}

fn read_msg(stream: &mut BufReader<TcpStream>) -> Result<JsonValue> {
    let mut len_bytes: [u8; 4] = [0; 4];
    stream.read_exact(&mut len_bytes)?;
    let len = u32::from_be_bytes(len_bytes) as usize;
    let mut bytes = vec![0; len];
    stream.read_exact(&mut bytes)?;
    let resp = String::from_utf8(bytes)?;
    let value = serde_json::from_str::<JsonValue>(&resp)?;
    if let Some(obj) = value.as_object()
        && let Some(status) = obj.get("status")
        && status == "err"
        && let Some(reason) = obj.get("reason")
    {
        bail!("musing error: {}", reason.to_string());
    }

    Ok(value)
}

fn write_msg(stream: &mut BufReader<TcpStream>, msg: JsonValue) -> Result<()> {
    let msg = msg.to_string();
    let bytes = msg.as_bytes();
    let len_bytes = (bytes.len() as u32).to_be_bytes();
    stream.get_mut().write_all(&len_bytes)?;
    stream.get_mut().write_all(bytes)?;

    Ok(())
}

pub fn metadata(
    stream: &mut BufReader<TcpStream>,
    paths: Vec<String>,
    tags: Option<Vec<String>>,
) -> Result<Vec<HashMap<String, String>>> {
    let mut request = Map::new();
    request.insert("kind".into(), "metadata".into());
    request.insert("paths".into(), paths.into());
    let _ = match tags {
        Some(tags) => request.insert("tags".into(), tags.into()),
        None => request.insert("all_tags".into(), true.into()),
    };
    write_msg(stream, JsonValue::from(request))?;
    let mut res = read_msg(stream)?;
    if let Some(obj) = res.as_object_mut()
        && let Some(mut metadata) = obj.remove("metadata")
        && let Some(metadata) = metadata.as_array_mut()
    {
        let metadata: Vec<_> = metadata
            .iter()
            .map(|v| match v.as_object() {
                Some(obj) => obj
                    .iter()
                    .map(|(k, v)| {
                        if v.is_string() {
                            (k.clone(), v.as_str().unwrap().to_string())
                        } else {
                            (k.clone(), String::from(constants::UNKNOWN))
                        }
                    })
                    .collect::<HashMap<_, _>>(),
                None => HashMap::new(),
            })
            .collect();

        Ok(metadata)
    } else {
        bail!("could not fetch metadata");
    }
}

pub fn grouped_songs(
    stream: &mut BufReader<TcpStream>,
    group_by: Vec<String>,
    children_tags: Vec<String>,
) -> Result<HashMap<Vec<String>, SongGroup>> {
    let mut request = Map::new();
    request.insert("kind".into(), "select".into());
    request.insert("tags".into(), children_tags.clone().into());
    request.insert("group_by".into(), group_by.clone().into());
    let comparators: Vec<_> = children_tags
        .iter()
        .map(|tag| json!({"tag": tag}))
        .collect();
    request.insert("comparators".into(), comparators.into());
    write_msg(stream, JsonValue::from(request))?;
    let mut res = read_msg(stream)?;
    if let Some(obj) = res.as_object_mut()
        && let Some(mut values) = obj.remove("values")
        && let Some(values) = values.as_array_mut()
    {
        let mut grouped = HashMap::new();
        for group in values.iter_mut().filter_map(|v| v.as_object_mut()) {
            // the combination of values id'ing these songs (e.g.: [albumartist, album])
            // the order of tags is the same as in the group_by argument
            let id_comb: Vec<_> = group_by
                .iter()
                .map(|tag| match group.remove(tag) {
                    Some(value) => value.as_str().unwrap_or(constants::UNKNOWN).to_string(),
                    None => constants::UNKNOWN.to_string(),
                })
                .collect();
            let (song_values, paths) = if let Some(songs) = group.remove("data")
                && let Some(songs) = songs.as_array()
            {
                let mut song_values = Vec::new();
                let mut paths = Vec::new();
                for song_data in songs {
                    if let Some(song_data) = song_data.as_array() {
                        let mut song_value = Vec::new();
                        for (_, value) in children_tags.iter().zip(song_data.iter()) {
                            song_value.push(value.as_str().map(|s| s.to_string()));
                        }
                        song_values.push(song_value);
                        paths.push(
                            song_data
                                .last()
                                .and_then(|s| s.as_str())
                                .unwrap_or(constants::UNKNOWN)
                                .to_string(),
                        );
                    }
                }

                (song_values, paths)
            } else {
                (Vec::new(), Vec::new())
            };
            grouped
                .entry(id_comb)
                .and_modify(|group: &mut SongGroup| {
                    group.add_songs(&children_tags, &song_values, &paths)
                })
                .or_insert(SongGroup::new(&children_tags, &song_values, &paths));
        }

        Ok(grouped)
    } else {
        bail!("could not select songs");
    }
}

pub fn state_delta(stream: &mut BufReader<TcpStream>) -> Result<MusingStateDelta> {
    let request = json!({
        "kind": "state",
    });
    write_msg(stream, request)?;
    let res = read_msg(stream)?;

    MusingStateDelta::try_from(res)
}

pub fn seek(stream: &mut BufReader<TcpStream>, seconds: i64) -> Result<()> {
    let request = json!({
        "kind": "seek",
        "seconds": seconds,
    });
    write_msg(stream, request)?;

    read_msg(stream).map(|_| ())
}

pub fn speed(stream: &mut BufReader<TcpStream>, delta: i16) -> Result<()> {
    let request = json!({
        "kind": "speed",
        "delta": delta,
    });
    write_msg(stream, request)?;

    read_msg(stream).map(|_| ())
}

pub fn volume(stream: &mut BufReader<TcpStream>, delta: i8) -> Result<()> {
    let request = json!({
        "kind": "volume",
        "delta": delta,
    });
    write_msg(stream, request)?;

    read_msg(stream).map(|_| ())
}

pub fn add_to_queue(stream: &mut BufReader<TcpStream>, paths: Vec<String>) -> Result<()> {
    let request = json!({"kind": "addqueue", "paths": paths});
    write_msg(stream, request)?;

    read_msg(stream).map(|_| ())
}

pub fn play(stream: &mut BufReader<TcpStream>, id: u64) -> Result<()> {
    let request = json!({
        "kind": "play",
        "id": id,
    });
    write_msg(stream, request)?;

    read_msg(stream).map(|_| ())
}

pub fn remove(stream: &mut BufReader<TcpStream>, id: u64) -> Result<()> {
    let request = json!({
        "kind": "removequeue",
        "ids": [id],
    });
    write_msg(stream, request)?;

    read_msg(stream).map(|_| ())
}

pub fn update(stream: &mut BufReader<TcpStream>) -> Result<String> {
    let request = json!({ "kind": "update" });
    write_msg(stream, request)?;

    let res = read_msg(stream)?;
    if let Some(obj) = res.as_object()
        && let Some(status) = obj.get("status")
        && status == "ok"
    {
        let added_songs = obj
            .get("added_songs")
            .and_then(|s| s.as_u64())
            .unwrap_or_default();
        let removed_songs = obj
            .get("removed_songs")
            .and_then(|s| s.as_u64())
            .unwrap_or_default();

        Ok(format!(
            "update successful, added {} songs, removed {} songs",
            added_songs, removed_songs
        ))
    } else {
        bail!("could not update the database");
    }
}

// a convenience function for sending requests that have neither
// any additional arguments nor a meaningful positive response
pub fn other(stream: &mut BufReader<TcpStream>, kind: String) -> Result<()> {
    let request = json!({
        "kind": kind,
    });
    write_msg(stream, request)?;

    read_msg(stream).map(|_| ())
}
