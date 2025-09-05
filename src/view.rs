use anyhow::Result;
use itertools::izip;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style, Styled, Stylize},
    symbols::{border, line},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Cell, LineGauge, Padding, Paragraph, Row, Table, TableState, Widget,
    },
};

use crate::{
    app::{App, AppState, Screen},
    constants,
};

fn render_header(app: &App, frame: &mut Frame, area: Rect) {
    let volume = app.musing_state.volume;
    let speed = app.musing_state.speed;
    let mode = app.musing_state.playback_mode;
    let state = app.musing_state.playback_state;
    let gapless = app.musing_state.gapless;
    let current = app.musing_state.current;
    let is_stopped = app.musing_state.is_stopped();
    let metadata = current.map(|cur| &app.queue_state.metadata[cur as usize]);
    let path = current.map(|cur| &app.musing_state.queue[cur as usize].path);

    let current_title = metadata
        .and_then(|m| m.get("tracktitle"))
        .map(|s| s.as_str())
        .or(path.map(|p| p.as_str()))
        .unwrap_or("<unknown title>");
    let current_artist = metadata
        .and_then(|m| m.get("artist"))
        .map(|s| s.as_str())
        .unwrap_or("<unknown artist>");
    let current_album = metadata
        .and_then(|m| m.get("album"))
        .map(|s| s.as_str())
        .unwrap_or("<unknown album>");

    let header = Table::default()
        .rows(vec![
            Row::new(vec![
                Cell::from(
                    Line::from(format!("[{} {}]", mode, if gapless { 'G' } else { 'g' }))
                        .left_aligned(),
                ),
                Cell::from(Line::from(if is_stopped { "" } else { current_title }).centered()),
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
    frame.render_widget(header, area);
}

fn render_footer(app: &App, frame: &mut Frame, area: Rect) {
    let timer = app.musing_state.timer;
    let elapsed = timer.map(|timer| timer.0).unwrap_or_default();
    let duration = timer.map(|timer| timer.1).unwrap_or_default();

    let footer = match app.status_msg.as_deref() {
        Some(msg) => Line::from(msg),
        None => {
            let timer_left = view_utils::format_time(elapsed);
            let timer_right = view_utils::format_time(duration);
            let progress_bar_width = (area.width as usize) - 2 * (timer_left.len() + 1);
            let done_part_width =
                (progress_bar_width as f32 * (elapsed as f32 / duration as f32)).round() as usize;

            Line::from(vec![
                Span::from(timer_left),
                format!(" {}", ".".repeat(done_part_width)).cyan(),
                format!("{} ", ".".repeat(progress_bar_width - done_part_width)).white(),
                Span::from(timer_right),
            ])
        }
    };
    frame.render_widget(footer, area);
}

fn render_cover_screen(app: &App, frame: &mut Frame) {
    // TODO: album cover goes here at some point (render with chafa)
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(frame.area());
    render_header(app, frame, layout[0]);
    // cover_art
    render_footer(app, frame, layout[2]);
}

fn render_queue_screen(app: &mut App, frame: &mut Frame) {
    let queue = &app.musing_state.queue;
    let metadata = &app.queue_state.metadata;
    let titles: Vec<_> = metadata
        .iter()
        .zip(queue.iter())
        .map(|(m, song)| {
            m.get("tracktitle")
                .map(|s| s.as_str())
                .unwrap_or(&song.path)
                .to_string()
        })
        .collect();
    let artists: Vec<_> = metadata
        .iter()
        .map(|m| {
            m.get("artist")
                .map(|s| s.as_str())
                .unwrap_or(constants::UNKNOWN)
                .to_string()
        })
        .collect();
    let albums: Vec<_> = metadata
        .iter()
        .map(|m| {
            m.get("album")
                .map(|s| s.as_str())
                .unwrap_or(constants::UNKNOWN)
                .to_string()
        })
        .collect();
    let durations_int: Vec<_> = metadata
        .iter()
        .map(|m| {
            m.get("duration")
                .map(|s| s.as_str().parse::<u64>().unwrap_or_default())
                .unwrap_or_default()
        })
        .collect();
    let total_duration = durations_int.iter().sum::<u64>();
    let durations: Vec<_> = durations_int
        .into_iter()
        .map(|d| view_utils::format_time(d))
        .collect();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(frame.area());
    let block = Block::default()
        .borders(Borders::ALL)
        .title(
            Line::from(format!(
                "Total duration: {}",
                view_utils::format_time(total_duration)
            ))
            .cyan(),
        )
        .title_alignment(Alignment::Center)
        .padding(Padding::horizontal(1));
    let rows: Vec<_> = izip!(titles, artists, albums, durations)
        .enumerate()
        .map(|(i, t)| {
            let v = vec![t.0, t.1, t.2, t.3];
            if app.musing_state.current.is_some_and(|cur| cur == i as u64) {
                Row::new(v).style(Style::default().blue())
            } else {
                Row::new(v)
            }
        })
        .collect();
    let list = Table::default()
        .rows(rows)
        .widths(vec![
            Constraint::Fill(4),
            Constraint::Fill(3),
            Constraint::Fill(2),
            Constraint::Fill(1),
        ])
        .block(block)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    render_header(app, frame, layout[0]);
    frame.render_stateful_widget(list, layout[1], &mut app.queue_state.state);
    render_footer(app, frame, layout[2]);
}

fn render_library_screen(app: &mut App, frame: &mut Frame) {}

pub fn render(app: &mut App, frame: &mut Frame) {
    // TODO: add theming (make a view struct with the theme)
    match app.screen {
        Screen::Cover => render_cover_screen(app, frame),
        Screen::Queue => render_queue_screen(app, frame),
        Screen::Library => render_library_screen(app, frame),
    }
}

pub mod view_utils {
    use super::*;

    pub fn format_time(secs: u64) -> String {
        format!("{:02}:{:02}", (secs / 60).min(99), secs % 60)
    }
}
