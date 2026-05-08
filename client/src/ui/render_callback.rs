use futures::future::ready;

use crate::{
    connection::{Connected, ConnectionEvent},
    ui::render_state::RenderState,
};

pub enum RenderCallback {
    RenderState(RenderState),
}

impl RenderCallback {
    pub async fn release<T: RenderBehaviour>(self, unit: &mut T) {
        match self {
            RenderCallback::RenderState(render_state) => {
                unit.handle_render_state(render_state).await
            }
        }
    }
}

pub trait RenderBehaviour {
    fn get_render_callback(&mut self) -> impl Future<Output = RenderCallback> {
        ready(RenderCallback::RenderState(RenderState::default()))
    }

    fn handle_render_state(&mut self, _render_state: RenderState) -> impl Future<Output = ()> {
        ready(())
    }

    fn send_render_callback(&mut self, render_callback: RenderCallback) -> impl Future<Output = ()>
    where
        Self: Connected,
    {
        async {
            self.connection()
                .send(ConnectionEvent::RenderCallback(render_callback))
                .await;
        }
    }
}
