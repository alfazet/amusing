use anyhow::{Result, bail};
use ratatui::style::{Color, Modifier, Style, Stylize};
use toml::{Table, Value as TomlValue};

#[derive(Debug)]
pub struct Theme {
    pub current_title: Style,
    pub current_artist: Style,
    pub current_album: Style,
    pub selection_primary: Style,
    pub selection_secondary: Style,
    pub search_box: Style,
    pub total_duration: Style,
    pub progress_bar_done: Style,
    pub progress_bar_rest: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            current_title: Style::default().fg(Color::Red).bold().italic(),
            current_artist: Style::default().fg(Color::LightMagenta).bold(),
            current_album: Style::default().fg(Color::LightMagenta).bold(),
            selection_primary: Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::REVERSED),
            selection_secondary: Style::default().fg(Color::Blue),
            search_box: Style::default().fg(Color::Blue),
            total_duration: Style::default().fg(Color::Cyan),
            progress_bar_done: Style::default().fg(Color::Cyan),
            progress_bar_rest: Style::default(),
        }
    }
}

impl TryFrom<Table> for Theme {
    type Error = anyhow::Error;

    fn try_from(table: Table) -> Result<Self> {
        let mut theme = Theme::default();
        for (key, val) in table {
            match (key.as_str(), val) {
                ("current_title", TomlValue::Table(current_title)) => {
                    theme.current_title = try_from_table(current_title)?;
                }
                ("current_artist", TomlValue::Table(current_artist)) => {
                    theme.current_artist = try_from_table(current_artist)?;
                }
                ("current_album", TomlValue::Table(current_album)) => {
                    theme.current_album = try_from_table(current_album)?;
                }
                ("selection_primary", TomlValue::Table(selection_primary)) => {
                    theme.selection_primary = try_from_table(selection_primary)?;
                }
                ("selection_secondary", TomlValue::Table(selection_secondary)) => {
                    theme.selection_secondary = try_from_table(selection_secondary)?;
                }
                ("search_box", TomlValue::Table(search_box)) => {
                    theme.search_box = try_from_table(search_box)?;
                }
                ("total_duration", TomlValue::Table(total_duration)) => {
                    theme.total_duration = try_from_table(total_duration)?;
                }
                ("progress_bar_done", TomlValue::Table(progress_bar_done)) => {
                    theme.progress_bar_done = try_from_table(progress_bar_done)?;
                }
                ("progress_bar_rest", TomlValue::Table(progress_bar_rest)) => {
                    theme.progress_bar_rest = try_from_table(progress_bar_rest)?;
                }
                (other, _) => bail!("invalid config key `{}`", other),
            }
        }

        Ok(theme)
    }
}

fn try_from_table(mut table: Table) -> Result<Style> {
    let add_modifier = table
        .get("add_modifier")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string())
        .unwrap_or_default();
    let sub_modifier = table
        .get("sub_modifier")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string())
        .unwrap_or_default();
    table.insert("add_modifier".into(), TomlValue::String(add_modifier));
    table.insert("sub_modifier".into(), TomlValue::String(sub_modifier));

    table.try_into().map_err(|e| e.into())
}
