use crossterm::event::{KeyCode, KeyEvent};
use futures::{SinkExt, StreamExt, stream::SplitSink};
use logger::{async_log, log_entry::LogEntry};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::{Line, Text},
    widgets::{Borders, Wrap},
};
use tokio::select;
use tokio_tungstenite::{WebSocketStream, connect_async, tungstenite::Message};
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::{
    connection::{Connected, Connection, ConnectionBehaviour},
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

pub struct Chat {
    connection: Connection,
    cancellation_token: CancellationToken,

    chat: Text<'static>,
    buffer: String,
    sender: Option<
        SplitSink<
            WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
            Message,
        >,
    >,
}

impl Chat {
    async fn handle_message(
        &mut self,
        maybe_message: Result<Message, tokio_tungstenite::tungstenite::Error>,
    ) {
        match maybe_message {
            Ok(message) => match message {
                Message::Text(buffer) => {
                    self.chat.lines.push(Line::from(""));
                    self.chat.lines.push(Line::from(buffer.clone()));
                    self.chat.lines.push(Line::from(""));

                    let render_state = self.get_render_state().await;
                    self.connection
                        .send(RenderCallback::RenderState(render_state).into())
                        .await;
                }
                _ => {}
            },
            Err(e) => {
                _ = async_log!(LogEntry::from(
                    format!("Chat handle message error: {:?}", e).as_bytes()
                ))
                .await;
            }
        }
    }

    async fn send_message(&mut self, message: String) {
        if let Some(sender) = self.sender.as_mut() {
            if let Err(e) = sender.send(Message::Text(message)).await {
                _ = async_log!(LogEntry::from(
                    format!("Chat send message error: {:?}", e).as_bytes()
                ))
                .await;
            }
        }
    }
}

impl Connected for Chat {
    fn connection(&mut self) -> &mut Connection {
        &mut self.connection
    }
}

impl ControlEventHook for Chat {}

impl RenderBehaviour for Chat {
    fn get_render_state(&mut self) -> impl Future<Output = RenderState> {
        async {
            RenderState::default()
                .layout(Layout::new(
                    Direction::Vertical,
                    [Constraint::Min(1), Constraint::Length(3)],
                ))
                .top_state(
                    TopState::default()
                        .text(self.chat.clone())
                        .style(match self.connection.render_sender_is_some() {
                            true => Style::new(),
                            false => Style::new().dark_gray(),
                        })
                        .borders(Borders::ALL)
                        .wrap(Wrap { trim: true })
                        .title("Chat"),
                )
        }
    }
}

impl InputEventBehaviour for Chat {
    fn key_event(&mut self, key_event: KeyEvent) -> impl Future<Output = ()> {
        async move {
            match key_event.code {
                KeyCode::Char(c) => self.buffer.push(c),
                KeyCode::Backspace => {
                    self.buffer.pop();
                }
                KeyCode::Enter => {
                    let mut buffer = String::new();
                    std::mem::swap(&mut self.buffer, &mut buffer);

                    self.chat.lines.push(Line::from(""));
                    self.chat.lines.push(Line::from(buffer.clone()));
                    self.chat.lines.push(Line::from(""));

                    self.send_message(buffer).await;

                    let render_state = self.get_render_state().await;
                    self.connection
                        .send(RenderCallback::RenderState(render_state).into())
                        .await;
                }
                _ => {}
            }

            let render_state = self.get_render_state().await;
            self.connection
                .send(RenderCallback::RenderState(render_state).into())
                .await;
        }
    }
}

impl Quit for Chat {
    fn cancellation_token(&self) -> &CancellationToken {
        &self.cancellation_token
    }
}

impl ConnectionBehaviour for Chat {}

impl Component for Chat {
    fn new(connection: Connection) -> Self {
        Self {
            connection,
            cancellation_token: CancellationToken::new(),
            chat: Text::default(),
            buffer: String::new(),
            sender: None,
        }
    }

    fn main_loop(mut self) -> impl Future<Output = ()> {
        async move {
            let (ws_stream, _) = match connect_async(
                Url::parse(&format!("ws://127.0.0.1:3000/ws")).unwrap(),
            )
            .await
            {
                Ok(ws_stream) => ws_stream,
                Err(e) => {
                    _ = async_log!(LogEntry::from(
                        format!("Chat start error: {:?}", e).as_bytes()
                    ))
                    .await;
                    return;
                }
            };

            let (tx, mut rx) = ws_stream.split();
            self.sender = Some(tx);

            loop {
                select! {
                    connection_event = self.connection.recv() => {
                        self.release(connection_event).await;
                    }
                    Some(message) = rx.next() => {
                        self.handle_message(message).await;
                    }
                    _ = self.cancellation_token.cancelled() => {
                        break;
                    }
                }
            }
        }
    }
}
