use crate::{
    controller::control_event::{ControlEvent, ControlEventHook},
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
    control_receiver: mpsc::Receiver<ControlEvent>,
}

impl Connection {
    pub fn new(control_receiver: mpsc::Receiver<ControlEvent>) -> Self {
        Self {
            render_sender: None,
            render_receiver: None,
            input_event_sender: None,
            input_event_receiver: None,
            control_receiver,
        }
    }

    pub fn set_render_sender(&mut self, render_sender: mpsc::Sender<RenderCallback>) {
        self.render_sender = Some(render_sender);
    }

    pub fn set_render_receiver(&mut self, render_receiver: mpsc::Receiver<RenderCallback>) {
        self.render_receiver = Some(render_receiver);
    }

    pub fn set_input_event_sender(&mut self, input_event_sender: mpsc::Sender<InputEvent>) {
        self.input_event_sender = Some(input_event_sender);
    }

    pub fn set_input_event_receiver(&mut self, input_event_receiver: mpsc::Receiver<InputEvent>) {
        self.input_event_receiver = Some(input_event_receiver);
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

            Some(control) = self.control_receiver.recv() => {
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
                            async_log!(LogEntry::from(
                                format!("Connection send input event error: {:?}", e).as_bytes()
                            ));
                            false
                        }
                    }
                } else {
                    false
                }
            }
            ConnectionEvent::ControlEvent(_control_event) => false,
            ConnectionEvent::RenderCallback(render_callback) => {
                if let Some(sender) = self.render_sender.as_mut() {
                    match sender.send(render_callback).await {
                        Ok(_) => true,
                        Err(_) => {
                            async_log!(LogEntry::from(
                                format!("Connection send render error").as_bytes()
                            ));
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
            async_log!(LogEntry::from(
                format!("Connection send error: {:?}", e).as_bytes()
            ));
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
                async_log!(LogEntry::from(
                    format!("Connection recv error: {:?}", e).as_bytes()
                ));
            }
        }
    }

    pub async fn send_input_event_channel(
        &mut self,
        bridge: oneshot::Sender<mpsc::Receiver<InputEvent>>,
    ) {
        if let Err(e) = bridge.send(self.input_event_receiver.take().unwrap()) {
            async_log!(LogEntry::from(
                format!("Connection send error: {:?}", e).as_bytes()
            ));
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
                async_log!(LogEntry::from(
                    format!("Connection recv error: {:?}", e).as_bytes()
                ));
            }
        }
    }
}

pub trait Connected {
    fn connection(&mut self) -> &mut Connection;
}

pub trait ConnectionBehaviour:
    ControlEventHook + InputEventBehaviour + RenderBehaviour + Connected
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
