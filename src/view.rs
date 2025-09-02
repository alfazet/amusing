use anyhow::Result;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Rect, Alignment},
    style::{Stylize, Color},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

use crate::app::{App, AppState, Screen};

fn render_state(app: &App, frame: &mut Frame, rect: Rect) {
    let volume = app.musing_state.volume;

    let line = Line::from(format!("Volume: {}%", volume)).style(Color::Cyan);
    let widget = Paragraph::new(line).alignment(Alignment::Center);
    frame.render_widget(widget, rect);
}

fn render_queue(app: &App, frame: &mut Frame) {}

fn render_library(app: &App, frame: &mut Frame) {}

pub fn render(app: &App, frame: &mut Frame) {
    // TODO: add theming (make a view struct with the theme)
    render_state(app, frame, frame.area());
    match app.screen {
        Screen::Queue => render_queue(app, frame),
        Screen::Library => render_library(app, frame),
    }
}
