use crossterm::event::{self, Event, KeyEvent};
use logger::{log_entry::LogEntry, sync_log_quiet};
use std::thread;
use tokio::sync::mpsc;

pub enum EventControl {
    SwapRenderTop,
    SwapRenderBottom,
    SwapEvent,
}

pub struct EventManager {
    event_channel: mpsc::Sender<KeyEvent>,
    event_control_channel: mpsc::Sender<EventControl>,
}

impl EventManager {
    pub fn new(
        event_channel: mpsc::Sender<KeyEvent>,
        event_control_channel: mpsc::Sender<EventControl>,
    ) -> Self {
        Self {
            event_channel,
            event_control_channel,
        }
    }

    pub async fn spawn_task(self) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            self.start();
        })
    }

    pub fn start(self) {
        loop {
            match event::poll(std::time::Duration::from_millis(100)) {
                Ok(true) => match event::read() {
                    Ok(evt) => self.send_event(evt),
                    Err(e) => {
                        sync_log_quiet!(LogEntry::from(
                            format!("UserManager error: {:?}", e).as_bytes()
                        ));
                    }
                },
                Ok(false) => {}
                Err(e) => {
                    sync_log_quiet!(LogEntry::from(
                        format!("UserManager error: {:?}", e).as_bytes()
                    ));
                }
            }
        }
    }

    pub fn send_event(&self, event: Event) {
        match event {
            Event::Key(key_event) => {
                if let Err(e) = self.event_channel.try_send(key_event) {
                    sync_log_quiet!(LogEntry::from(
                        format!("UserManager send error: {:?}", e).as_bytes()
                    ));
                }
            }
            _ => {}
        }
    }
}
