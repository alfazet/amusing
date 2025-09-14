use anyhow::Result;
use base64::prelude::*;
use image::ImageReader;
use ratatui_image::{
    picker::Picker,
    thread::{ResizeRequest, ThreadProtocol},
};
use std::{io::Cursor, sync::mpsc as std_chan, thread};

use crate::event_handler::Event;

pub struct CoverArtState {
    pub picker: Picker,
    pub state: ThreadProtocol,
    pub draw: bool,
}

impl CoverArtState {
    pub fn try_new(tx_resize: std_chan::Sender<Event>) -> Result<Self> {
        let (tx, rx) = std_chan::channel::<ResizeRequest>();
        thread::spawn(move || {
            loop {
                if let Ok(request) = rx.recv() {
                    let _ = tx_resize.send(Event::CoverArtResize(request.resize_encode()));
                }
            }
        });
        let picker = Picker::from_query_stdio()?;
        let state = ThreadProtocol::new(tx, None);

        Ok(Self {
            picker,
            state,
            draw: false,
        })
    }

    pub fn replace_art(&mut self, new_data: Option<impl AsRef<str>>) -> Result<()> {
        match new_data {
            Some(new_data) => {
                let bytes = BASE64_STANDARD.decode(new_data.as_ref().as_bytes())?;
                let image = ImageReader::new(Cursor::new(bytes))
                    .with_guessed_format()?
                    .decode()?;
                self.state
                    .replace_protocol(self.picker.new_resize_protocol(image));
                self.draw = true;
            }
            None => self.draw = false,
        }

        Ok(())
    }
}
