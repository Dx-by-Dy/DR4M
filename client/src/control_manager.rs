use std::collections::HashMap;

use crossterm::event::KeyEvent;
use tokio::sync::{mpsc, oneshot};

use crate::{event_manager::EventControl, ui_manager::RenderCallback};

pub enum ControlEvent {
    SendRenderChannel(oneshot::Sender<mpsc::Sender<RenderCallback>>),
    RecvRenderChannel(oneshot::Receiver<mpsc::Sender<RenderCallback>>),
    SendEventChannel(oneshot::Sender<mpsc::Receiver<KeyEvent>>),
    RecvEventChannel(oneshot::Receiver<mpsc::Receiver<KeyEvent>>),
}

impl ControlEvent {
    pub async fn release<T: ControlEventBehaviour>(self, unit: &mut T) {
        match self {
            ControlEvent::SendRenderChannel(sender) => {
                unit.send_render_channel(sender).await;
            }
            ControlEvent::RecvRenderChannel(receiver) => {
                unit.recv_render_channel(receiver).await;
            }
            ControlEvent::SendEventChannel(sender) => {
                unit.send_event_channel(sender).await;
            }
            ControlEvent::RecvEventChannel(receiver) => {
                unit.recv_event_channel(receiver).await;
            }
        }
    }
}

pub trait ControlEventBehaviour {
    fn send_render_channel(
        &mut self,
        bridge: oneshot::Sender<mpsc::Sender<RenderCallback>>,
    ) -> impl Future<Output = ()>;
    fn recv_render_channel(
        &mut self,
        bridge: oneshot::Receiver<mpsc::Sender<RenderCallback>>,
    ) -> impl Future<Output = ()>;
    fn send_event_channel(
        &mut self,
        bridge: oneshot::Sender<mpsc::Receiver<KeyEvent>>,
    ) -> impl Future<Output = ()>;
    fn recv_event_channel(
        &mut self,
        bridge: oneshot::Receiver<mpsc::Receiver<KeyEvent>>,
    ) -> impl Future<Output = ()>;
}

struct ControlManager {
    managers_channels: HashMap<String, mpsc::Sender<oneshot::Sender<ControlEvent>>>,
    event_control_channel: mpsc::Receiver<EventControl>,
}

impl ControlManager {
    pub fn new(event_control_channel: mpsc::Receiver<EventControl>) -> Self {
        Self {
            managers_channels: HashMap::new(),
            event_control_channel,
        }
    }

    // pub async fn send_callback(&self, render_callback: Box<dyn FnOnce(&mut Frame)>) {
    //     self.tx.send(render_callback);
    // }
}
