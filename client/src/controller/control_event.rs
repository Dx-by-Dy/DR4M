use crate::{
    connection::Connected, controller::component::Quit, inputter::input_event::InputEvent,
    ui::render_callback::RenderCallback,
};
use futures::future::ready;
use tokio::sync::{mpsc, oneshot};

pub enum ToComponentEvent {
    SendRenderChannel(oneshot::Sender<mpsc::Sender<RenderCallback>>),
    RecvRenderChannel(oneshot::Receiver<mpsc::Sender<RenderCallback>>),
    SendInputEventChannel(oneshot::Sender<mpsc::Receiver<InputEvent>>),
    RecvInputEventChannel(oneshot::Receiver<mpsc::Receiver<InputEvent>>),
}

pub enum ToControllerEvent {
    SwapRenderChannel,
    SwapInputEventChannel,
}

pub enum ControlEvent {
    ToComponentEvent(ToComponentEvent),
    ToControllerEvent(ToControllerEvent),
    Quit,
}

impl ControlEvent {
    pub async fn release<T: ControlEventHook + Connected + Quit>(self, unit: &mut T) {
        match self {
            ControlEvent::ToComponentEvent(event) => match event {
                ToComponentEvent::SendRenderChannel(sender) => {
                    unit.send_render_hook().await;
                    unit.connection().send_render_channel(sender).await;
                }
                ToComponentEvent::RecvRenderChannel(receiver) => {
                    unit.connection().recv_render_channel(receiver).await;
                    unit.recv_render_hook().await;
                }
                ToComponentEvent::SendInputEventChannel(sender) => {
                    unit.send_input_event_hook().await;
                    unit.connection().send_input_event_channel(sender).await;
                }
                ToComponentEvent::RecvInputEventChannel(receiver) => {
                    unit.connection().recv_input_event_channel(receiver).await;
                    unit.recv_input_event_hook().await;
                }
            },
            ControlEvent::ToControllerEvent(event) => {
                unit.to_controller_event_hook(event).await;
            }
            ControlEvent::Quit => {
                unit.quit_hook().await;
                unit.cancellation_token().cancel();
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
    fn quit_hook(&mut self) -> impl Future<Output = ()> {
        ready(())
    }
    fn to_controller_event_hook(
        &mut self,
        _to_controller_event: ToControllerEvent,
    ) -> impl Future<Output = ()> {
        ready(())
    }
}

impl From<ToComponentEvent> for ControlEvent {
    fn from(to_component_event: ToComponentEvent) -> Self {
        ControlEvent::ToComponentEvent(to_component_event)
    }
}

impl From<ToControllerEvent> for ControlEvent {
    fn from(to_controller_event: ToControllerEvent) -> Self {
        ControlEvent::ToControllerEvent(to_controller_event)
    }
}
