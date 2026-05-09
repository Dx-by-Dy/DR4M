use crate::{
    connection::{Connected, Connection, ConnectionBehaviour, ConnectionEvent},
    controller::{
        component::{Component, Quit},
        control_event::{ControlEvent, ControlEventHook},
    },
    inputter::input_event::InputEventBehaviour,
    ui::{
        render_callback::{RenderBehaviour, RenderCallback},
        render_state::{RenderState, bottom_state::BottomState},
    },
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Offset},
    style::Style,
    text::Text,
    widgets::{Borders, Wrap},
};
use tokio::select;
use tokio_util::sync::CancellationToken;

pub struct Commander {
    connection: Connection,
    buffer: String,
    cancellation_token: CancellationToken,
}

impl InputEventBehaviour for Commander {
    fn key_event(&mut self, key_event: KeyEvent) -> impl Future<Output = ()> {
        async move {
            match key_event.code {
                KeyCode::Char(c) => self.buffer.push(c),
                KeyCode::Backspace => {
                    self.buffer.pop();
                }
                KeyCode::Enter => {}
                KeyCode::Esc => {
                    self.connection
                        .send(ConnectionEvent::ControlEvent(ControlEvent::Quit))
                        .await;
                }
                _ => {}
            }

            let render_state = self.get_render_state().await;
            self.connection
                .send(ConnectionEvent::RenderCallback(
                    RenderCallback::RenderState(render_state),
                ))
                .await;
        }
    }
}

impl RenderBehaviour for Commander {
    fn get_render_state(&mut self) -> impl Future<Output = RenderState> {
        async {
            RenderState::default()
                .layout(Layout::new(
                    Direction::Vertical,
                    [Constraint::Min(1), Constraint::Length(3)],
                ))
                .bottom_state(
                    BottomState::default()
                        .text(Text::from(self.buffer.clone()))
                        .style(match self.connection.render_sender_is_some() {
                            true => Style::new(),
                            false => Style::new().dark_gray(),
                        })
                        .borders(Borders::ALL)
                        .wrap(Wrap { trim: true })
                        .title("Commander")
                        .cursor_offset(Offset::new(1 + self.buffer.len() as i32, 1)),
                )
        }
    }
}

impl Connected for Commander {
    fn connection(&mut self) -> &mut Connection {
        &mut self.connection
    }
}

impl ControlEventHook for Commander {
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

impl ConnectionBehaviour for Commander {}

impl Quit for Commander {
    fn cancellation_token(&self) -> &CancellationToken {
        &self.cancellation_token
    }
}

impl Component for Commander {
    fn new(connection: Connection) -> Self {
        Self {
            connection,
            buffer: String::new(),
            cancellation_token: CancellationToken::new(),
        }
    }

    fn main_loop(mut self) -> impl Future<Output = ()> + Send + 'static {
        async move {
            loop {
                select! {
                    connection_event = self.connection.recv() => {
                        self.release(connection_event).await;
                    }
                    _ = self.cancellation_token.cancelled() => {
                        break;
                    }
                }
            }
        }
    }
}
