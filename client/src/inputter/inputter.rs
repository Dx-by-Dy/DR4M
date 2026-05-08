use crate::inputter::input_event::InputEvent;
use crossterm::event::{self, Event};
use logger::{log_entry::LogEntry, sync_log_quiet};
use std::thread;
use tokio::sync::mpsc;

pub struct Inputter {
    input_event_channel: mpsc::Sender<InputEvent>,
}

impl Inputter {
    pub fn new(input_event_channel: mpsc::Sender<InputEvent>) -> Self {
        Self {
            input_event_channel,
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
                if let Err(e) = self
                    .input_event_channel
                    .try_send(InputEvent::KeyEvent(key_event))
                {
                    sync_log_quiet!(LogEntry::from(
                        format!("UserManager send error: {:?}", e).as_bytes()
                    ));
                }
            }
            Event::FocusGained => todo!(),
            Event::FocusLost => todo!(),
            Event::Mouse(_) => todo!(),
            Event::Paste(_) => todo!(),
            Event::Resize(_, _) => todo!(),
        }
    }
}
