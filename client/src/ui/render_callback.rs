use futures::future::ready;
use ratatui::Frame;

pub enum RenderCallback {
    Render(Box<dyn FnOnce(&mut Frame<'_>) + Send + 'static>),
    Redraw,
}

impl RenderCallback {
    pub async fn release<T: RenderCallbackBehaviour>(self, unit: &mut T) {
        match self {
            RenderCallback::Render(render_callback) => unit.render(render_callback).await,
            RenderCallback::Redraw => unit.redraw().await,
        }
    }
}

pub trait RenderCallbackBehaviour {
    fn render(
        &mut self,
        _render_callback: Box<dyn FnOnce(&mut Frame<'_>) + Send + 'static>,
    ) -> impl Future<Output = ()> {
        ready(())
    }
    fn redraw(&mut self) -> impl Future<Output = ()> {
        ready(())
    }
}
