use ratatui::{
    style::Style,
    text::Text,
    widgets::{Block, Borders, Paragraph, Wrap},
};

#[derive(Default)]
pub struct BottomState {
    pub text: Text<'static>,
    pub wrap: Wrap,
    pub borders: Borders,
    pub style: Style,
    pub title: &'static str,
}

impl BottomState {
    pub fn text(mut self, text: Text<'static>) -> Self {
        self.text = text;
        self
    }

    pub fn wrap(mut self, wrap: Wrap) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn borders(mut self, borders: Borders) -> Self {
        self.borders = borders;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn title(mut self, title: &'static str) -> Self {
        self.title = title;
        self
    }
}

impl Into<Paragraph<'static>> for BottomState {
    fn into(self) -> Paragraph<'static> {
        Paragraph::new(self.text).wrap(self.wrap).block(
            Block::default()
                .style(self.style)
                .borders(self.borders)
                .title(self.title),
        )
    }
}
