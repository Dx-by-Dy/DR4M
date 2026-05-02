use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use logger::{log_entry::LogEntry, sync_log_quiet};
use ratatui::{Terminal, prelude::CrosstermBackend};

pub struct UI {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl UI {
    pub fn new() -> Result<Self, std::io::Error> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        sync_log_quiet!(LogEntry::from("UI initialized".as_bytes()));

        Ok(Self {
            terminal: Terminal::new(CrosstermBackend::new(stdout))?,
        })
    }
}

impl Drop for UI {
    fn drop(&mut self) {
        _ = disable_raw_mode();
        _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
    }
}
