use crate::{
    connection::{Connected, Connection, ConnectionBehaviour},
    controller::control_event::ControlEventHook,
    inputter::input_event::InputEventBehaviour,
    ui::render_callback::{RenderCallback, RenderCallbackBehaviour},
};
use crossterm::event::KeyEvent;
use logger::{async_log_quiet, async_read_log, log_entry::LogEntry};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use tokio::{select, task::JoinHandle};

pub struct LoggerManager {
    connection: Connection,
    buffer: Text<'static>,
}

impl LoggerManager {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection,
            buffer: Text::default(),
        }
    }

    pub async fn spawn_task(self) -> JoinHandle<()> {
        tokio::spawn(self.start())
    }

    async fn start(mut self) {
        loop {
            select! {
                connection_event = self.connection.recv() => {
                    self.release(connection_event).await;
                }
                Ok(log_event) = async_read_log!() => {
                    self.handle_log_event(log_event).await;
                }
            }
        }
    }

    async fn handle_log_event(&mut self, event: LogEntry) {
        self.buffer.lines.push(Line::from(format!("{}", event)));
        self.render().await;
    }

    async fn render(&mut self) {
        let style = match self.connection.render_sender_is_some() {
            true => Style::new(),
            false => Style::new().dark_gray(),
        };
        let paragraph = Paragraph::new(self.buffer.clone())
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(style)
                    .title("Logger"),
            );
        let render = Box::new(move |frame: &mut Frame<'_>| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(3)])
                .split(frame.area());
            frame.render_widget(paragraph, chunks[0]);
        });
        if let Err(_) = self
            .connection
            .render_try_send(RenderCallback::Render(render))
            .await
        {
            async_log_quiet!(LogEntry::from(
                format!("LoggerManager send error").as_bytes()
            ))
            .await;
        }
    }

    fn handle_key_event(&mut self, _event: KeyEvent) {}
}

impl ControlEventHook for LoggerManager {
    fn recv_render_hook(&mut self) -> impl Future<Output = ()> {
        self.render()
    }
}

impl InputEventBehaviour for LoggerManager {
    fn key_event(&mut self, key_event: KeyEvent) -> impl Future<Output = ()> {
        async move { self.handle_key_event(key_event) }
    }
}

impl RenderCallbackBehaviour for LoggerManager {}

impl Connected for LoggerManager {
    fn connection(&mut self) -> &mut Connection {
        &mut self.connection
    }
}

impl ConnectionBehaviour for LoggerManager {}
