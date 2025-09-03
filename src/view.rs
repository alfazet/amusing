use anyhow::Result;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Cell, Paragraph, Row, Table, Widget},
};

use crate::app::{App, AppState, Screen};

fn render_cover_screen(app: &App, frame: &mut Frame) {
    // TODO: album cover goes here at some point (render with chafa)
    let volume = app.musing_state.volume;
    let speed = app.musing_state.speed;
    let mode = app.musing_state.playback_mode;
    let is_stopped = app.musing_state.is_stopped();
    let gapless = app.musing_state.gapless;
    let current = app.musing_state.current;

    let metadata = current.map(|i| &app.metadata[i as usize]);
    let current_title = metadata
        .and_then(|m| m.get("tracktitle"))
        .map(|s| s.as_str())
        .unwrap_or("<unknown title>");
    let current_artist = metadata
        .and_then(|m| m.get("artist"))
        .map(|s| s.as_str())
        .unwrap_or("<unknown artist>");
    let current_album = metadata
        .and_then(|m| m.get("album"))
        .map(|s| s.as_str())
        .unwrap_or("<unknown album>");

    // row 0, left column: [w e r g], (sequential, single, random, gapless) - the active ones are
    // capitalized
    // just below that a [paused], [stopped] or [playing]
    let header = Table::default()
        .rows(vec![
            Row::new(vec![
                Cell::from(Line::from(format!("[{}]", mode)).left_aligned()),
                Cell::from(
                    Line::from(if is_stopped {
                        "[musing stopped]"
                    } else {
                        current_title
                    })
                    .centered(),
                ),
                Cell::from(Line::from(format!("volume: {}", volume)).right_aligned()),
            ]),
            Row::new(vec![
                Cell::from(Line::from(if gapless { "[gapless]" } else { "" }).left_aligned()),
                Cell::from(
                    Line::from(if is_stopped {
                        "".into()
                    } else {
                        format!("{} - {}", current_artist, current_album)
                    })
                    .centered(),
                ),
                Cell::from(Line::from(format!("speed: {}", speed)).right_aligned()),
            ]),
        ])
        .widths(vec![
            Constraint::Max(12),
            Constraint::Min(24),
            Constraint::Max(12),
        ]);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(15),
            Constraint::Percentage(65),
            Constraint::Percentage(20),
        ])
        .split(frame.area());

    // at the bottom smth like this
    // 0:00 -------------------- 3:40
    // with the dashes progressively changing color
    // if there's a status msg then it replaces that line for a couple
    // of seconds

    frame.render_widget(header, layout[0]);
}

fn render_queue_screen(app: &App, frame: &mut Frame) {}

fn render_library_screen(app: &App, frame: &mut Frame) {}

pub fn render(app: &App, frame: &mut Frame) {
    // TODO: add theming (make a view struct with the theme)
    match app.screen {
        Screen::Cover => render_cover_screen(app, frame),
        Screen::Queue => render_queue_screen(app, frame),
        Screen::Library => render_library_screen(app, frame),
    }
}
