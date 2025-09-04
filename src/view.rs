use anyhow::Result;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Stylize},
    symbols::{border, line},
    text::{Line, Span, Text},
    widgets::{Block, Cell, LineGauge, Paragraph, Row, Table, Widget},
};

use crate::app::{App, AppState, Screen};

fn render_cover_screen(app: &App, frame: &mut Frame) {
    // TODO: album cover goes here at some point (render with chafa)
    let volume = app.musing_state.volume;
    let speed = app.musing_state.speed;
    let mode = app.musing_state.playback_mode;
    let state = app.musing_state.playback_state;
    let gapless = app.musing_state.gapless;
    let current = app.musing_state.current;
    let timer = app.musing_state.timer;
    let is_stopped = app.musing_state.is_stopped();

    let metadata = current.map(|cur| &app.metadata[cur as usize]);
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
    let elapsed = timer.map(|timer| timer.0).unwrap_or_default();
    let duration = timer.map(|timer| timer.1).unwrap_or_default();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(frame.area());
    let header = Table::default()
        .rows(vec![
            Row::new(vec![
                Cell::from(
                    Line::from(format!("[{} {}]", mode, if gapless { 'G' } else { 'g' }))
                        .left_aligned(),
                ),
                Cell::from(
                    Line::from(if is_stopped {
                        "[musing stopped]"
                    } else {
                        current_title
                    })
                    .centered(),
                ),
                Cell::from(Line::from(format!("Volume: {}", volume)).right_aligned()),
            ]),
            Row::new(vec![
                Cell::from(Line::from(format!("[{}]", state)).left_aligned()),
                Cell::from(
                    Line::from(if is_stopped {
                        "".into()
                    } else {
                        format!("{} - {}", current_artist, current_album)
                    })
                    .centered(),
                ),
                Cell::from(Line::from(format!("Speed: {}", speed)).right_aligned()),
            ]),
        ])
        .widths(vec![
            Constraint::Length(12),
            Constraint::Fill(1),
            Constraint::Length(12),
        ]);

    let timer_left = format!("{:02}:{:02}", (elapsed / 60).min(99), elapsed % 60);
    let timer_right = format!("{:02}:{:02}", (duration / 60).min(99), duration % 60);
    let progress_bar_width = (layout[2].width as usize) - 2 * (timer_left.len() + 1);
    let done_part_width =
        (progress_bar_width as f32 * (elapsed as f32 / duration as f32)).round() as usize;
    let progress_bar = Line::from(vec![
        Span::from(timer_left),
        format!(" {}", ".".repeat(done_part_width)).cyan(),
        format!("{} ", ".".repeat(progress_bar_width - done_part_width)).white(),
        Span::from(timer_right),
    ]);

    frame.render_widget(header, layout[0]);
    frame.render_widget(progress_bar, layout[2]);
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
