use crate::{
    connection::{Connected, Connection, ConnectionBehaviour, ConnectionEvent},
    controller::{
        component::{Component, Quit},
        control_event::ControlEventHook,
    },
    inputter::input_event::InputEventBehaviour,
    ui::{
        render_callback::{RenderBehaviour, RenderCallback},
        render_state::{RenderState, top_state::TopState},
    },
};
use logger::{async_log, async_read_log, log_entry::LogEntry};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::{Line, Text},
    widgets::{Borders, Wrap},
};
use tokio::select;
use tokio_util::sync::CancellationToken;

pub struct LogReader {
    connection: Connection,
    buffer: Text<'static>,
    cancellation_token: CancellationToken,
}

impl LogReader {
    async fn handle_log_event(&mut self, event: LogEntry) {
        self.buffer.lines.push(Line::from(format!("{}", event)));
        let render_state = self.get_render_state().await;
        self.connection
            .send(ConnectionEvent::RenderCallback(
                RenderCallback::RenderState(render_state),
            ))
            .await;
    }
}

impl ControlEventHook for LogReader {
    fn recv_render_hook(&mut self) -> impl Future<Output = ()> {
        async {
            let render_state = self.get_render_state().await;
            self.connection
                .send(ConnectionEvent::RenderCallback(
                    RenderCallback::RenderState(render_state),
                ))
                .await;
        }
    }
}

impl InputEventBehaviour for LogReader {}

impl RenderBehaviour for LogReader {
    fn get_render_state(&mut self) -> impl Future<Output = RenderState> {
        async {
            RenderState::default()
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
                )
        }
    }
}

impl Connected for LogReader {
    fn connection(&mut self) -> &mut Connection {
        &mut self.connection
    }
}

impl ConnectionBehaviour for LogReader {}

impl Quit for LogReader {
    fn cancellation_token(&self) -> &CancellationToken {
        &self.cancellation_token
    }
}

impl Component for LogReader {
    fn new(connection: Connection) -> Self {
        Self {
            connection,
            buffer: Text::default(),
            cancellation_token: CancellationToken::new(),
        }
    }

    fn main_loop(mut self) -> impl Future<Output = ()> {
        async move {
            _ = async_log!(LogEntry::from(format!("LogReader start").as_bytes())).await;

            let render_state = self.get_render_state().await;
            self.connection
                .send(RenderCallback::RenderState(render_state).into())
                .await;

            loop {
                select! {
                    connection_event = self.connection.recv() => {
                        self.release(connection_event).await;
                    }
                    Ok(log_event) = async_read_log!() => {
                        self.handle_log_event(log_event).await;
                    }
                    _ = self.cancellation_token.cancelled() => {
                        break;
                    }
                }
            }
        }
    }
}
