use crate::{
    controller::{
        component::Quit,
        control_event::{ControlEvent, ControlEventHook},
    },
    inputter::input_event::{InputEvent, InputEventBehaviour},
    ui::render_callback::{RenderBehaviour, RenderCallback},
};
use logger::{async_log, log_entry::LogEntry};
use tokio::{
    select,
    sync::{mpsc, oneshot},
};

pub enum ConnectionEvent {
    InputEvent(InputEvent),
    RenderCallback(RenderCallback),
    ControlEvent(ControlEvent),
}

pub struct Connection {
    render_sender: Option<mpsc::Sender<RenderCallback>>,
    render_receiver: Option<mpsc::Receiver<RenderCallback>>,
    input_event_sender: Option<mpsc::Sender<InputEvent>>,
    input_event_receiver: Option<mpsc::Receiver<InputEvent>>,
    control_sender: Option<mpsc::Sender<ControlEvent>>,
    control_receiver: Option<mpsc::Receiver<ControlEvent>>,
}

impl Connection {
    pub fn new() -> Self {
        Self {
            render_sender: None,
            render_receiver: None,
            input_event_sender: None,
            input_event_receiver: None,
            control_sender: None,
            control_receiver: None,
        }
    }

    pub fn set_render_sender(mut self, render_sender: mpsc::Sender<RenderCallback>) -> Self {
        self.render_sender = Some(render_sender);
        self
    }

    pub fn set_render_receiver(mut self, render_receiver: mpsc::Receiver<RenderCallback>) -> Self {
        self.render_receiver = Some(render_receiver);
        self
    }

    pub fn set_input_event_sender(mut self, input_event_sender: mpsc::Sender<InputEvent>) -> Self {
        self.input_event_sender = Some(input_event_sender);
        self
    }

    pub fn set_input_event_receiver(
        mut self,
        input_event_receiver: mpsc::Receiver<InputEvent>,
    ) -> Self {
        self.input_event_receiver = Some(input_event_receiver);
        self
    }

    pub fn set_control_sender(mut self, control_sender: mpsc::Sender<ControlEvent>) -> Self {
        self.control_sender = Some(control_sender);
        self
    }

    pub fn set_control_receiver(mut self, control_receiver: mpsc::Receiver<ControlEvent>) -> Self {
        self.control_receiver = Some(control_receiver);
        self
    }

    pub fn render_sender_is_some(&self) -> bool {
        self.render_sender.is_some()
    }

    pub fn render_receiver_is_some(&self) -> bool {
        self.render_receiver.is_some()
    }

    pub fn input_event_sender_is_some(&self) -> bool {
        self.input_event_sender.is_some()
    }

    pub fn input_event_receiver_is_some(&self) -> bool {
        self.input_event_receiver.is_some()
    }

    pub fn control_sender_is_some(&self) -> bool {
        self.control_sender.is_some()
    }
}

impl Connection {
    pub async fn recv(&mut self) -> ConnectionEvent {
        select! {
            Some(event) = async {
                if let Some(receiver) = self.input_event_receiver.as_mut() {
                    receiver.recv().await
                } else {
                    futures::future::pending().await
                }
            } => {
                ConnectionEvent::InputEvent(event)
            }

            Some(render) = async {
                if let Some(receiver) = self.render_receiver.as_mut() {
                    receiver.recv().await
                } else {
                    futures::future::pending().await
                }
            } => {
                ConnectionEvent::RenderCallback(render)
            }

            Some(control) = async {
                if let Some(receiver) = self.control_receiver.as_mut() {
                    receiver.recv().await
                } else {
                    futures::future::pending().await
                }
            } => {
                ConnectionEvent::ControlEvent(control)
            }
        }
    }

    pub async fn send(&mut self, event: ConnectionEvent) -> bool {
        match event {
            ConnectionEvent::InputEvent(input_event) => {
                if let Some(sender) = self.input_event_sender.as_mut() {
                    match sender.send(input_event).await {
                        Ok(_) => true,
                        Err(e) => {
                            _ = async_log!(LogEntry::from(
                                format!("Connection send input event error: {:?}", e).as_bytes()
                            ))
                            .await;
                            false
                        }
                    }
                } else {
                    false
                }
            }
            ConnectionEvent::ControlEvent(control_event) => {
                if let Some(sender) = self.control_sender.as_mut() {
                    match sender.send(control_event).await {
                        Ok(_) => true,
                        Err(e) => {
                            _ = async_log!(LogEntry::from(
                                format!("Connection send control event error: {:?}", e).as_bytes()
                            ))
                            .await;
                            false
                        }
                    }
                } else {
                    false
                }
            }
            ConnectionEvent::RenderCallback(render_callback) => {
                if let Some(sender) = self.render_sender.as_mut() {
                    match sender.send(render_callback).await {
                        Ok(_) => true,
                        Err(_) => {
                            _ = async_log!(LogEntry::from(
                                format!("Connection send render error").as_bytes()
                            ))
                            .await;
                            false
                        }
                    }
                } else {
                    false
                }
            }
        }
    }
}

impl Connection {
    pub async fn send_render_channel(
        &mut self,
        bridge: oneshot::Sender<mpsc::Sender<RenderCallback>>,
    ) {
        if let Err(e) = bridge.send(self.render_sender.take().unwrap()) {
            _ = async_log!(LogEntry::from(
                format!("Connection send error: {:?}", e).as_bytes()
            ))
            .await;
        }
    }

    pub async fn recv_render_channel(
        &mut self,
        bridge: oneshot::Receiver<mpsc::Sender<RenderCallback>>,
    ) {
        match bridge.await {
            Ok(channel) => {
                self.render_sender = Some(channel);
            }
            Err(e) => {
                _ = async_log!(LogEntry::from(
                    format!("Connection recv error: {:?}", e).as_bytes()
                ))
                .await;
            }
        }
    }

    pub async fn send_input_event_channel(
        &mut self,
        bridge: oneshot::Sender<mpsc::Receiver<InputEvent>>,
    ) {
        if let Err(e) = bridge.send(self.input_event_receiver.take().unwrap()) {
            _ = async_log!(LogEntry::from(
                format!("Connection send error: {:?}", e).as_bytes()
            ))
            .await;
        }
    }

    pub async fn recv_input_event_channel(
        &mut self,
        bridge: oneshot::Receiver<mpsc::Receiver<InputEvent>>,
    ) {
        match bridge.await {
            Ok(channel) => {
                self.input_event_receiver = Some(channel);
            }
            Err(e) => {
                _ = async_log!(LogEntry::from(
                    format!("Connection recv error: {:?}", e).as_bytes()
                ))
                .await;
            }
        }
    }
}

pub trait Connected {
    fn connection(&mut self) -> &mut Connection;
}

pub trait ConnectionBehaviour:
    ControlEventHook + InputEventBehaviour + RenderBehaviour + Connected + Quit
{
    fn release(&mut self, event: ConnectionEvent) -> impl Future<Output = ()> {
        async {
            match event {
                ConnectionEvent::InputEvent(input_event) => input_event.release(self).await,
                ConnectionEvent::ControlEvent(control_event) => control_event.release(self).await,
                ConnectionEvent::RenderCallback(render_callback) => {
                    render_callback.release(self).await
                }
            }
        }
    }
}

impl From<ControlEvent> for ConnectionEvent {
    fn from(control_event: ControlEvent) -> Self {
        ConnectionEvent::ControlEvent(control_event)
    }
}

impl From<RenderCallback> for ConnectionEvent {
    fn from(render_callback: RenderCallback) -> Self {
        ConnectionEvent::RenderCallback(render_callback)
    }
}

impl From<InputEvent> for ConnectionEvent {
    fn from(input_event: InputEvent) -> Self {
        ConnectionEvent::InputEvent(input_event)
    }
}
