use anyhow::Error;
use ratatui::style::{Color, Style, Stylize};

#[derive(Debug)]
pub struct Theme {
    pub current_title: Style,
    pub current_artist: Style,
    pub current_album: Style,
    pub selection: Style,
    pub progress_bar_done: (char, Style),
    pub progress_bar_rem: (char, Style),
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            current_title: Style::default().fg(Color::Red).bold(),
            current_artist: Style::default().fg(Color::LightRed).bold(),
            current_album: Style::default().fg(Color::LightRed),
            selection: Style::default().fg(Color::Blue),
            progress_bar_done: ('.', Style::default().fg(Color::Cyan)),
            progress_bar_rem: ('.', Style::default().fg(Color::Cyan)),
        }
    }
}
