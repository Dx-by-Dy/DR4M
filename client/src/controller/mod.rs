pub mod component;
pub mod control_event;

use crate::{
    commander::Commander,
    connection::{Connected, Connection, ConnectionBehaviour},
    controller::{
        component::{Component, Quit},
        control_event::{ControlEvent, ControlEventHook},
    },
    inputter::{Inputter, input_event::InputEventBehaviour},
    ui::{UI, render_callback::RenderBehaviour},
};
use std::collections::HashMap;
use tokio::{select, sync::mpsc};
use tokio_util::sync::CancellationToken;

pub struct Controller {
    connection: Connection,
    control_channels: HashMap<String, mpsc::Sender<ControlEvent>>,
    cancellation_token: CancellationToken,
}

impl Controller {
    pub fn new() -> Self {
        let (controller_control_sender, controller_control_receiver) = mpsc::channel(1024);
        let (commander_control_sender, commander_control_receiver) = mpsc::channel(1024);
        let (inputter_control_sender, inputter_control_receiver) = mpsc::channel(1024);
        let (ui_control_sender, ui_control_receiver) = mpsc::channel(1024);

        let (input_event_sender, input_event_receiver) = mpsc::channel(1024);
        let (render_event_sender, render_event_receiver) = mpsc::channel(1024);

        let controller_connection =
            Connection::new().set_control_receiver(controller_control_receiver);

        let commander_connection = Connection::new()
            .set_control_receiver(commander_control_receiver)
            .set_input_event_receiver(input_event_receiver)
            .set_render_sender(render_event_sender)
            .set_control_sender(controller_control_sender);

        let inputter_connection = Connection::new()
            .set_control_receiver(inputter_control_receiver)
            .set_input_event_sender(input_event_sender);

        let ui_connection = Connection::new()
            .set_control_receiver(ui_control_receiver)
            .set_render_receiver(render_event_receiver);

        let mut control_channels = HashMap::new();
        control_channels.insert("commander".to_string(), commander_control_sender);
        control_channels.insert("inputter".to_string(), inputter_control_sender);
        control_channels.insert("ui".to_string(), ui_control_sender);

        Commander::start(commander_connection);
        Inputter::start(inputter_connection);
        UI::start(ui_connection);

        Self {
            connection: controller_connection,
            control_channels,
            cancellation_token: CancellationToken::new(),
        }
    }
}

impl Component for Controller {
    fn new(_connection: Connection) -> Self {
        Self::new()
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

impl ConnectionBehaviour for Controller {}

impl ControlEventHook for Controller {
    fn quit_hook(&mut self) -> impl Future<Output = ()> {
        async {
            for (_, control_channel) in self.control_channels.iter() {
                let _ = control_channel.send(ControlEvent::Quit).await;
            }
        }
    }
}

impl RenderBehaviour for Controller {}

impl InputEventBehaviour for Controller {}

impl Connected for Controller {
    fn connection(&mut self) -> &mut Connection {
        &mut self.connection
    }
}

impl Quit for Controller {
    fn cancellation_token(&self) -> &CancellationToken {
        &self.cancellation_token
    }
}
