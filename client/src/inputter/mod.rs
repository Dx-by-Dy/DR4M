pub mod input_event;

use crate::{
    connection::{Connected, Connection, ConnectionBehaviour, ConnectionEvent},
    controller::{
        component::{Component, Quit},
        control_event::{ControlEvent, ControlEventHook, ToControllerEvent},
    },
    inputter::input_event::{InputEvent, InputEventBehaviour},
    ui::render_callback::RenderBehaviour,
};
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers};
use futures::StreamExt;
use tokio::select;
use tokio_util::sync::CancellationToken;

pub struct Inputter {
    connection: Connection,
    cancellation_token: CancellationToken,
}

impl Inputter {
    async fn handle_event(&mut self, event: Event) {
        if event == Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)) {
            self.connection.send(ControlEvent::Quit.into()).await;
            return;
        }

        if event == Event::Key(KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE)) {
            self.connection
                .send(ConnectionEvent::ControlEvent(
                    ControlEvent::ToControllerEvent(ToControllerEvent::Swap),
                ))
                .await;
            return;
        }

        match event {
            Event::Key(key_event) => {
                self.connection
                    .send(ConnectionEvent::InputEvent(InputEvent::KeyEvent(key_event)))
                    .await;
            }
            Event::FocusGained => {}
            Event::FocusLost => {}
            Event::Mouse(_) => {}
            Event::Paste(_) => {}
            Event::Resize(_, _) => {}
        }
    }
}

impl Component for Inputter {
    fn new(connection: Connection) -> Self {
        Self {
            connection,
            cancellation_token: CancellationToken::new(),
        }
    }

    fn main_loop(mut self) -> impl Future<Output = ()> + Send + 'static {
        async move {
            let mut event_stream = EventStream::new();

            loop {
                select! {
                    connection_event = self.connection.recv() => {
                        self.release(connection_event).await;
                    }
                    Some(Ok(event)) = event_stream.next() => {
                        self.handle_event(event).await;
                    }
                    _ = self.cancellation_token.cancelled() => {
                        break;
                    }
                }
            }
        }
    }
}

impl ControlEventHook for Inputter {}

impl InputEventBehaviour for Inputter {}

impl RenderBehaviour for Inputter {}

impl Connected for Inputter {
    fn connection(&mut self) -> &mut Connection {
        &mut self.connection
    }
}

impl Quit for Inputter {
    fn cancellation_token(&self) -> &CancellationToken {
        &self.cancellation_token
    }
}

impl ConnectionBehaviour for Inputter {}
