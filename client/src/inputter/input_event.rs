use crossterm::event::KeyEvent;
use futures::future::ready;

pub enum InputEvent {
    KeyEvent(KeyEvent),
}

impl InputEvent {
    pub async fn release<T: InputEventBehaviour>(self, unit: &mut T) {
        match self {
            InputEvent::KeyEvent(key_event) => unit.key_event(key_event).await,
        }
    }
}

pub trait InputEventBehaviour: Sized {
    fn key_event(&mut self, _key_event: KeyEvent) -> impl Future<Output = ()> {
        ready(())
    }
}
