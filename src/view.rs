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
        Block, Borders, Cell, LineGauge, List, Padding, Paragraph, Row, Table, TableState, Widget,
    },
};

use crate::{
    app::{App, AppState, Screen},
    constants,
    model::{
        common::FocusedPart,
        search::{Search, SearchState},
    },
};

const SEARCH_PROMPT: &str = "> ";

fn render_header(app: &App, frame: &mut Frame, area: Rect) {
    let volume = app.musing_state.volume;
    let speed = app.musing_state.speed;
    let mode = app.musing_state.playback_mode;
    let state = app.musing_state.playback_state;
    let gapless = app.musing_state.gapless;
    let current = app.musing_state.current;
    let is_stopped = app.musing_state.is_stopped();
    let metadata = current.and_then(|cur| app.queue_state.metadata.get(cur as usize));
    let path = current
        .and_then(|cur| app.musing_state.queue.get(cur as usize))
        .map(|x| &x.path);

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

fn render_search_box(app: &App, frame: &mut Frame, area: Rect, search: &Search) {
    let cursor_pos = search.input.visual_cursor();
    frame.set_cursor_position((
        area.x + SEARCH_PROMPT.len() as u16 + cursor_pos as u16 + 1,
        area.y + 1,
    ));
    let search_block = if let SearchState::Idle = search.state {
        Block::default().borders(Borders::ALL)
    } else {
        Block::default()
            .borders(Borders::ALL)
            .border_style(Color::Blue)
    };
    let search_box =
        Paragraph::new(format!("{}{}", SEARCH_PROMPT, search.input.value())).block(search_block);
    frame.render_widget(search_box, area);
}

fn render_cover_screen(app: &mut App, frame: &mut Frame) {
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
    let metadata = app.queue_state.ordered_metadata();
    let displayed_data: &Vec<_> = &metadata
        .iter()
        .map(|m| {
            app.queue_state
                .displayed_tags
                .iter()
                .map(|tag| {
                    m.get(tag)
                        .map(|s| s.as_str())
                        .unwrap_or(constants::UNKNOWN)
                        .to_string()
                })
                .collect::<Vec<_>>()
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
        .map(view_utils::format_time)
        .collect();

    let rows: Vec<_> = displayed_data
        .iter()
        .zip(durations)
        .enumerate()
        .map(|(i, t)| {
            let mut v = t.0.clone();
            v.push(t.1);
            if app
                .musing_state
                .current
                .is_some_and(|cur| cur == app.queue_state.search.real_i(i) as u64)
            {
                Row::new(v).style(Style::default().fg(Color::Blue))
            } else {
                Row::new(v)
            }
        })
        .collect();
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
    let list = Table::default()
        .rows(rows)
        .widths(
            (1..=(&(app.queue_state.displayed_tags.len() as u16) + 1))
                .rev()
                .map(Constraint::Fill),
        )
        .block(block)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(frame.area());
    render_header(app, frame, layout[0]);
    let search = &app.queue_state.search;
    match search.state {
        SearchState::Off => {
            frame.render_stateful_widget(list, layout[1], &mut app.queue_state.state)
        }
        _ => {
            let sublayout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Fill(1), Constraint::Length(3)])
                .split(layout[1]);
            frame.render_stateful_widget(list, sublayout[0], &mut app.queue_state.state);
            render_search_box(app, frame, sublayout[1], search);
        }
    }
    render_footer(app, frame, layout[2]);
}

fn render_library_screen(app: &mut App, frame: &mut Frame) {
    let default_highlight = Style::default().add_modifier(Modifier::REVERSED);
    let (child_highlight, song_highlight) = match &app.library_state.focused_part {
        FocusedPart::Groups => (default_highlight.fg(Color::Blue), default_highlight),
        FocusedPart::Child(_) => (default_highlight, default_highlight.fg(Color::Blue)),
    };

    let children: Vec<_> = app
        .library_state
        .ordered_children()
        .iter()
        .map(|child| Row::new(child.id_comb.clone()))
        .collect();
    let children_block = Block::default()
        .borders(Borders::ALL)
        .title_alignment(Alignment::Center)
        .padding(Padding::horizontal(1));
    let children_list = Table::default()
        .rows(children)
        .widths(vec![Constraint::Fill(1), Constraint::Fill(1)])
        .block(children_block)
        .row_highlight_style(child_highlight);

    let songs = app
        .library_state
        .selected_child()
        .map(|child| {
            let group = child.ordered_group();
            let mut titles = Vec::new();
            for (meta, path) in group {
                titles.push(meta.get("tracktitle").unwrap_or(path).to_string());
            }

            titles
        })
        .unwrap_or_default();
    let songs_block = Block::default()
        .borders(Borders::ALL)
        .title_alignment(Alignment::Center)
        .padding(Padding::horizontal(1));
    let song_list = Table::default()
        .rows(songs.into_iter().map(|song| Row::new([song])))
        .block(songs_block)
        .row_highlight_style(song_highlight);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(frame.area());
    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Fill(1), Constraint::Fill(1)])
        .split(layout[1]);
    render_header(app, frame, layout[0]);
    let lhs_search = &app.library_state.search;
    match lhs_search.state {
        SearchState::Off => {
            frame.render_stateful_widget(children_list, middle[0], &mut app.library_state.state)
        }
        _ => {
            let sublayout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Fill(1), Constraint::Length(3)])
                .split(middle[0]);
            frame.render_stateful_widget(children_list, sublayout[0], &mut app.library_state.state);
            render_search_box(app, frame, sublayout[1], lhs_search);
        }
    }
    if let Some(child) = app.library_state.selected_child() {
        let rhs_search = &child.search;
        match rhs_search.state {
            SearchState::Off => {
                let state = &mut app.library_state.selected_child_mut().unwrap().state;
                frame.render_stateful_widget(song_list, middle[1], state);
            }
            _ => {
                let sublayout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![Constraint::Fill(1), Constraint::Length(3)])
                    .split(middle[1]);
                render_search_box(app, frame, sublayout[1], rhs_search);
                let state = &mut app.library_state.selected_child_mut().unwrap().state;
                frame.render_stateful_widget(song_list, sublayout[0], state);
            }
        }
    }
    render_footer(app, frame, layout[2]);
}

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
        let h = secs / 3600;
        let m = (secs - h * 3600) / 60;
        let s = secs - h * 3600 - m * 60;
        if h == 0 {
            format!("{:02}:{:02}", m, s)
        } else {
            format!("{:02}:{:02}:{:02}", h, m, s)
        }
    }
}
