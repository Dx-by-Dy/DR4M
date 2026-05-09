pub mod bottom_state;
pub mod top_state;

use crate::ui::render_state::{bottom_state::BottomState, top_state::TopState};
use ratatui::{
    Frame,
    layout::{Layout, Position},
    widgets::Paragraph,
};

#[derive(Default)]
pub struct RenderState {
    pub top_state: Option<TopState>,
    pub bottom_state: Option<BottomState>,
    pub layout: Option<Layout>,
}

impl RenderState {
    pub fn render_frame(self, frame: &mut Frame<'_>) {
        if let Some(layout) = self.layout {
            let chunks = layout.split(frame.area());
            if let Some(top_state) = self.top_state {
                let offset = top_state.cursor_offset;
                frame.render_widget(Paragraph::from(top_state.into()), chunks[0]);
                frame.set_cursor_position(Position::new(chunks[0].x, chunks[0].y).offset(offset));
            }
            if let Some(bottom_state) = self.bottom_state {
                let offset = bottom_state.cursor_offset;
                frame.render_widget(Paragraph::from(bottom_state.into()), chunks[1]);
                frame.set_cursor_position(Position::new(chunks[1].x, chunks[1].y).offset(offset));
            }
        }
    }
}

impl RenderState {
    pub fn top_state(mut self, top_state: TopState) -> Self {
        self.top_state = Some(top_state);
        self
    }

    pub fn bottom_state(mut self, bottom_state: BottomState) -> Self {
        self.bottom_state = Some(bottom_state);
        self
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = Some(layout);
        self
    }
}
