pub mod input_event;

use crate::{
    connection::{Connected, Connection, ConnectionBehaviour, ConnectionEvent},
    controller::{component::Component, control_event::ControlEventHook},
    inputter::input_event::{InputEvent, InputEventBehaviour},
    ui::render_callback::RenderBehaviour,
};
use crossterm::event::{Event, EventStream};
use futures::StreamExt;
use tokio::select;

pub struct Inputter {
    connection: Connection,
}

impl Inputter {
    async fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key_event) => {
                self.connection
                    .send(ConnectionEvent::InputEvent(InputEvent::KeyEvent(key_event)))
                    .await;
            }
            Event::FocusGained => todo!(),
            Event::FocusLost => todo!(),
            Event::Mouse(_) => todo!(),
            Event::Paste(_) => todo!(),
            Event::Resize(_, _) => todo!(),
        }
    }
}

impl Component for Inputter {
    fn new(connection: Connection) -> Self {
        Self { connection }
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

impl ConnectionBehaviour for Inputter {}
