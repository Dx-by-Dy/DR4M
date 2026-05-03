use ratatui::Frame;
use tokio::sync::mpsc;

struct ChatManager {
    tx: mpsc::Sender<Box<dyn FnOnce(&mut Frame)>>,
}

impl ChatManager {
    pub fn new(tx: mpsc::Sender<Box<dyn FnOnce(&mut Frame)>>) -> Self {
        Self { tx }
    }

    pub async fn send_callback(&self, render_callback: Box<dyn FnOnce(&mut Frame)>) {
        self.tx.send(render_callback);
    }
}
