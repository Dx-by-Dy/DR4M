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
    pub cursor_position: Option<Position>,
}

impl RenderState {
    pub fn render_frame(self, frame: &mut Frame<'_>) {
        if let Some(layout) = self.layout {
            let chunks = layout.split(frame.area());
            if let Some(top_state) = self.top_state {
                frame.render_widget(Paragraph::from(top_state.into()), chunks[0]);
            }
            if let Some(bottom_state) = self.bottom_state {
                frame.render_widget(Paragraph::from(bottom_state.into()), chunks[1]);
            }
        }
        if let Some(cursor_position) = self.cursor_position {
            frame.set_cursor_position(cursor_position);
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

    pub fn cursor_position(mut self, cursor_position: Position) -> Self {
        self.cursor_position = Some(cursor_position);
        self
    }
}
