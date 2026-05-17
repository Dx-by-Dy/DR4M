pub mod component;
pub mod control_event;

use crate::{
    // chat::Chat,
    commander::Commander,
    connection::{Connected, Connection, ConnectionBehaviour},
    controller::{
        component::{Component, Quit},
        control_event::{ControlEvent, ControlEventHook, ToComponentEvent, ToControllerEvent},
    },
    inputter::{Inputter, input_event::InputEventBehaviour},
    log_reader::LogReader,
    ui::{UI, render_callback::RenderBehaviour},
};
use logger::{async_log, log_entry::LogEntry};
use tokio::{
    select,
    sync::{mpsc, oneshot},
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RenderFocus {
    Top,
    Bottom,
}

pub struct Controller {
    connection: Connection,
    cancellation_token: CancellationToken,

    render_focus: RenderFocus,
    top_render_queue: Vec<mpsc::Sender<ControlEvent>>,
    top_render_queue_index: usize,
    bottom_render_queue: Vec<mpsc::Sender<ControlEvent>>,
    bottom_render_queue_index: usize,
    input_event_queue: Vec<mpsc::Sender<ControlEvent>>,
    input_event_queue_index: usize,
    services: Vec<mpsc::Sender<ControlEvent>>,
}

impl Controller {
    pub fn new() -> Self {
        let (control_sender, control_receiver) = mpsc::channel(1024);
        let (commander_control_sender, commander_control_receiver) = mpsc::channel(1024);
        let (inputter_control_sender, inputter_control_receiver) = mpsc::channel(1024);
        let (ui_control_sender, ui_control_receiver) = mpsc::channel(1024);
        //let (chat_control_sender, chat_control_receiver) = mpsc::channel(1024);
        let (log_reader_control_sender, log_reader_control_receiver) = mpsc::channel(1024);

        let (input_event_sender, input_event_receiver) = mpsc::channel(1024);
        let (render_event_sender, render_event_receiver) = mpsc::channel(1024);

        let controller_connection = Connection::new().set_control_receiver(control_receiver);

        let commander_connection = Connection::new()
            .set_control_receiver(commander_control_receiver)
            .set_input_event_receiver(input_event_receiver)
            .set_render_sender(render_event_sender.clone())
            .set_control_sender(control_sender.clone());

        let inputter_connection = Connection::new()
            .set_control_receiver(inputter_control_receiver)
            .set_input_event_sender(input_event_sender)
            .set_control_sender(control_sender);

        let ui_connection = Connection::new()
            .set_control_receiver(ui_control_receiver)
            .set_render_receiver(render_event_receiver);

        //let chat_connection = Connection::new().set_control_receiver(chat_control_receiver);

        let log_reader_connection = Connection::new()
            .set_control_receiver(log_reader_control_receiver)
            .set_render_sender(render_event_sender.clone());

        Commander::start(commander_connection);
        Inputter::start(inputter_connection);
        UI::start(ui_connection);
        LogReader::start(log_reader_connection);
        //Chat::start(chat_connection);

        Self {
            connection: controller_connection,
            render_focus: RenderFocus::Bottom,
            top_render_queue: vec![
                log_reader_control_sender.clone(),
                //chat_control_sender.clone(),
            ],
            top_render_queue_index: 0,
            bottom_render_queue: vec![
                commander_control_sender.clone(),
                //chat_control_sender.clone(),
            ],
            bottom_render_queue_index: 0,
            input_event_queue: vec![
                commander_control_sender,
                log_reader_control_sender,
                //chat_control_sender,
            ],
            input_event_queue_index: 0,
            services: vec![inputter_control_sender, ui_control_sender],
            cancellation_token: CancellationToken::new(),
        }
    }

    async fn swap_top_render_channel(&mut self) {
        let (sender_component, receiver_component) = (
            &self.top_render_queue[self.top_render_queue_index],
            &self.top_render_queue[(self.top_render_queue_index + 1) % self.top_render_queue.len()],
        );

        let (oneshot_sender, oneshot_receiver) = oneshot::channel();
        if let Err(e) = sender_component
            .send(ControlEvent::ToComponentEvent(
                ToComponentEvent::SendRenderChannel(oneshot_sender),
            ))
            .await
        {
            _ = async_log!(LogEntry::from(
                format!("Controller send render channel error: {:?}", e).as_bytes()
            ))
            .await;
            return;
        }
        if let Err(e) = receiver_component
            .send(ControlEvent::ToComponentEvent(
                ToComponentEvent::RecvRenderChannel(oneshot_receiver),
            ))
            .await
        {
            _ = async_log!(LogEntry::from(
                format!("Controller recv render channel error: {:?}", e).as_bytes()
            ))
            .await;
            return;
        }

        self.top_render_queue_index =
            (self.top_render_queue_index + 1) % self.top_render_queue.len();
    }

    async fn swap_bottom_render_channel(&mut self) {
        let (sender_component, receiver_component) = (
            &self.bottom_render_queue[self.bottom_render_queue_index],
            &self.bottom_render_queue
                [(self.bottom_render_queue_index + 1) % self.bottom_render_queue.len()],
        );

        let (oneshot_sender, oneshot_receiver) = oneshot::channel();
        if let Err(e) = sender_component
            .send(ControlEvent::ToComponentEvent(
                ToComponentEvent::SendRenderChannel(oneshot_sender),
            ))
            .await
        {
            _ = async_log!(LogEntry::from(
                format!("Controller send render channel error: {:?}", e).as_bytes()
            ))
            .await;
            return;
        }
        if let Err(e) = receiver_component
            .send(ControlEvent::ToComponentEvent(
                ToComponentEvent::RecvRenderChannel(oneshot_receiver),
            ))
            .await
        {
            _ = async_log!(LogEntry::from(
                format!("Controller recv render channel error: {:?}", e).as_bytes()
            ))
            .await;
            return;
        }

        self.bottom_render_queue_index =
            (self.bottom_render_queue_index + 1) % self.bottom_render_queue.len();
    }

    async fn swap_input_event_channel(&mut self) {
        let (sender_component, receiver_component) = (
            &self.input_event_queue[self.input_event_queue_index],
            &self.input_event_queue
                [(self.input_event_queue_index + 1) % self.input_event_queue.len()],
        );

        let (oneshot_sender, oneshot_receiver) = oneshot::channel();
        if let Err(e) = sender_component
            .send(ControlEvent::ToComponentEvent(
                ToComponentEvent::SendInputEventChannel(oneshot_sender),
            ))
            .await
        {
            _ = async_log!(LogEntry::from(
                format!("Controller send input event channel error: {:?}", e).as_bytes()
            ))
            .await;
            return;
        }
        if let Err(e) = receiver_component
            .send(ControlEvent::ToComponentEvent(
                ToComponentEvent::RecvInputEventChannel(oneshot_receiver),
            ))
            .await
        {
            _ = async_log!(LogEntry::from(
                format!("Controller recv input event channel error: {:?}", e).as_bytes()
            ))
            .await;
            return;
        }

        self.input_event_queue_index =
            (self.input_event_queue_index + 1) % self.input_event_queue.len();
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
    fn to_controller_event_hook(
        &mut self,
        to_controller_event: ToControllerEvent,
    ) -> impl Future<Output = ()> {
        async move {
            match to_controller_event {
                ToControllerEvent::Swap => match self.render_focus {
                    RenderFocus::Top => {
                        self.swap_top_render_channel().await;
                    }
                    RenderFocus::Bottom => {
                        self.swap_bottom_render_channel().await;
                        self.swap_input_event_channel().await;
                    }
                },
            }
        }
    }

    fn quit_hook(&mut self) -> impl Future<Output = ()> {
        async {
            for control_channel in self.services.iter() {
                let _ = control_channel.send(ControlEvent::Quit).await;
            }

            for control_channel in self.top_render_queue.iter() {
                let _ = control_channel.send(ControlEvent::Quit).await;
            }

            for control_channel in self.bottom_render_queue.iter() {
                let _ = control_channel.send(ControlEvent::Quit).await;
            }

            for control_channel in self.input_event_queue.iter() {
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
