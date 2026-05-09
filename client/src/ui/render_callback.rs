use crate::ui::render_state::RenderState;
use futures::future::ready;

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
    fn get_render_state(&mut self) -> impl Future<Output = RenderState> {
        ready(RenderState::default())
    }

    fn handle_render_state(&mut self, _render_state: RenderState) -> impl Future<Output = ()> {
        ready(())
    }
}
