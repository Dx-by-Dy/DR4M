use crate::{
    connection::Connected, inputter::input_event::InputEvent, ui::render_callback::RenderCallback,
};
use futures::future::ready;
use tokio::sync::{mpsc, oneshot};

pub enum ControlEvent {
    SendRenderChannel(oneshot::Sender<mpsc::Sender<RenderCallback>>),
    RecvRenderChannel(oneshot::Receiver<mpsc::Sender<RenderCallback>>),
    SendInputEventChannel(oneshot::Sender<mpsc::Receiver<InputEvent>>),
    RecvInputEventChannel(oneshot::Receiver<mpsc::Receiver<InputEvent>>),
}

impl ControlEvent {
    pub async fn release<T: ControlEventHook + Connected>(self, unit: &mut T) {
        match self {
            ControlEvent::SendRenderChannel(sender) => {
                unit.send_render_hook().await;
                unit.connection().send_render_channel(sender).await;
            }
            ControlEvent::RecvRenderChannel(receiver) => {
                unit.connection().recv_render_channel(receiver).await;
                unit.recv_render_hook().await;
            }
            ControlEvent::SendInputEventChannel(sender) => {
                unit.send_input_event_hook().await;
                unit.connection().send_input_event_channel(sender).await;
            }
            ControlEvent::RecvInputEventChannel(receiver) => {
                unit.connection().recv_input_event_channel(receiver).await;
                unit.recv_input_event_hook().await;
            }
        }
    }
}

pub trait ControlEventHook {
    fn send_render_hook(&mut self) -> impl Future<Output = ()> {
        ready(())
    }
    fn recv_render_hook(&mut self) -> impl Future<Output = ()> {
        ready(())
    }
    fn send_input_event_hook(&mut self) -> impl Future<Output = ()> {
        ready(())
    }
    fn recv_input_event_hook(&mut self) -> impl Future<Output = ()> {
        ready(())
    }
}
