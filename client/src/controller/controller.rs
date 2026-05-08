use crate::{controller::control_event::ControlEvent, inputter::input_event::InputEvent};
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};

pub struct Controller {
    managers_channels: HashMap<String, mpsc::Sender<oneshot::Sender<ControlEvent>>>,
    input_event_receiver: mpsc::Receiver<InputEvent>,
    input_event_sender: mpsc::Sender<InputEvent>,
}

impl Controller {
    // pub fn new() -> Self {
    //     Self {
    //         managers_channels: HashMap::new(),
    //         event_control_channel,
    //     }
    // }

    // pub async fn send_callback(&self, render_callback: Box<dyn FnOnce(&mut Frame)>) {
    //     self.tx.send(render_callback);
    // }
}
