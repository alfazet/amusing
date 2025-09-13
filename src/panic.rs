use crate::constants;
use ratatui::crossterm::{ExecutableCommand, terminal};
use std::{backtrace::Backtrace, fs, io::stdout, panic};

pub fn register_backtrace_panic_handler() {
    panic::set_hook(Box::new(|info| {
        let _ = terminal::disable_raw_mode();
        let _ = stdout().execute(terminal::LeaveAlternateScreen);
        eprintln!("amusing crashed");
        if let Some(path) = dirs::cache_dir().map(|p| p.join(constants::DEFAULT_BACKTRACE_FILE)) {
            let _ = fs::write(
                &path,
                format!("{}\n{}", Backtrace::force_capture(), info).as_bytes(),
            );
            eprintln!("backtrace saved to `{}`", path.to_string_lossy());
        }
    }));
}
