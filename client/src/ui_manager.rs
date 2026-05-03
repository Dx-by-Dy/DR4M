use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures::io;
use logger::{async_log_quiet, log_entry::LogEntry};
use ratatui::{
    Frame, Terminal,
    layout::{Constraint, Direction, Layout, Position},
    prelude::CrosstermBackend,
    widgets::Paragraph,
};
use tokio::sync::mpsc;

pub enum RenderCallback {
    High(Paragraph<'static>),
    Bottom((Paragraph<'static>, Position)),
    Redraw,
}

pub struct UIManager {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    render_channel: mpsc::Receiver<RenderCallback>,
}

impl UIManager {
    pub fn new(render_channel: mpsc::Receiver<RenderCallback>) -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        Ok(Self {
            terminal: Terminal::new(CrosstermBackend::new(stdout))?,
            render_channel,
        })
    }

    pub async fn spawn_task(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            self.start().await;
        })
    }

    async fn start(mut self) {
        async_log_quiet!(LogEntry::from("UIManager started".as_bytes()));
        loop {
            if let Some(render_callback) = self.render_channel.recv().await {
                self.update(render_callback).await;
                async_log_quiet!(LogEntry::from("UIManager update".as_bytes()));
            }
        }
    }

    async fn update(&mut self, render_callback: RenderCallback) {
        match render_callback {
            RenderCallback::High(paragraph) => {
                if let Err(e) = self.terminal.draw(|frame: &mut Frame<'_>| {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Min(1), Constraint::Length(3)])
                        .split(frame.area());
                    frame.render_widget(paragraph, chunks[0]);
                }) {
                    async_log_quiet!(LogEntry::from(
                        format!("UIManager update error: {:?}", e).as_bytes()
                    ));
                }
            }
            RenderCallback::Bottom((paragraph, position)) => {
                if let Err(e) = self.terminal.draw(|frame| {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Min(1), Constraint::Length(3)])
                        .split(frame.area());
                    frame.render_widget(paragraph, chunks[1]);
                    frame.set_cursor_position(position);
                }) {
                    async_log_quiet!(LogEntry::from(
                        format!("UIManager update error: {:?}", e).as_bytes()
                    ));
                }
            }
            RenderCallback::Redraw => unimplemented!(),
        }
    }
}

impl Drop for UIManager {
    fn drop(&mut self) {
        _ = disable_raw_mode();
        _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
    }
}
