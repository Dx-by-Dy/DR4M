use crate::{
    connection::{Connected, Connection, ConnectionBehaviour},
    controller::{component::Component, control_event::ControlEventHook},
    inputter::input_event::InputEventBehaviour,
    ui::{
        render_callback::{RenderBehaviour, RenderCallback},
        render_state::{RenderState, top_state::TopState},
    },
};
use crossterm::event::KeyEvent;
use logger::{async_read_log, log_entry::LogEntry};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::{Line, Text},
    widgets::{Borders, Wrap},
};
use tokio::select;

pub struct LogReader {
    connection: Connection,
    buffer: Text<'static>,
}

impl LogReader {
    async fn handle_log_event(&mut self, event: LogEntry) {
        self.buffer.lines.push(Line::from(format!("{}", event)));
        let render_callback = self.get_render_callback().await;
        self.send_render_callback(render_callback).await;
    }
}

impl ControlEventHook for LogReader {
    fn recv_render_hook(&mut self) -> impl Future<Output = ()> {
        async {
            let render_callback = self.get_render_callback().await;
            self.send_render_callback(render_callback).await;
        }
    }
}

impl InputEventBehaviour for LogReader {
    fn key_event(&mut self, _key_event: KeyEvent) -> impl Future<Output = ()> {
        async move { todo!() }
    }
}

impl RenderBehaviour for LogReader {
    fn get_render_callback(&mut self) -> impl Future<Output = RenderCallback> {
        async {
            let render_state = RenderState::default()
                .layout(Layout::new(
                    Direction::Vertical,
                    [Constraint::Min(1), Constraint::Length(3)],
                ))
                .top_state(
                    TopState::default()
                        .text(self.buffer.clone())
                        .style(match self.connection.render_sender_is_some() {
                            true => Style::new(),
                            false => Style::new().dark_gray(),
                        })
                        .borders(Borders::ALL)
                        .wrap(Wrap { trim: true })
                        .title("Logger"),
                );
            RenderCallback::RenderState(render_state)
        }
    }
}

impl Connected for LogReader {
    fn connection(&mut self) -> &mut Connection {
        &mut self.connection
    }
}

impl ConnectionBehaviour for LogReader {}

impl Component for LogReader {
    fn new(connection: Connection) -> Self {
        Self {
            connection,
            buffer: Text::default(),
        }
    }

    fn main_loop(mut self) -> impl Future<Output = ()> {
        async move {
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
    }
}
