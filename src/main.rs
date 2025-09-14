use anyhow::Result;
use clap::Parser;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::{
        ExecutableCommand,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};
use std::io::stdout;

mod app;
mod config;
mod constants;
mod event_handler;
mod panic;
mod update;
mod view;

mod model;

use crate::{
    app::App,
    config::{CliOptions, Config},
};

fn run(config: Config) -> Result<()> {
    let mut app = App::try_new(config)?;
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let res = app.run(&mut terminal);
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    res
}

fn main() {
    let log_file = dirs::cache_dir().unwrap().join("amusing.log");
    let _ = simple_logging::log_to_file(log_file, log::LevelFilter::Warn);
    panic::register_backtrace_panic_handler();
    let cli_opts = CliOptions::parse();
    let config = match Config::try_from_file(cli_opts.config_file.as_deref()) {
        Ok(config) => config.merge_with_cli(cli_opts),
        Err(e) => {
            log::warn!("issue with loading config ({}), falling back to default", e);
            Config::default().merge_with_cli(cli_opts)
        }
    };

    if let Err(e) = run(config) {
        eprintln!("fatal error ({})", e);
    }
}
