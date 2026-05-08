use ratatui::{
    style::Style,
    text::Text,
    widgets::{Block, Borders, Paragraph, Wrap},
};

#[derive(Default)]
pub struct TopState {
    pub text: Option<Text<'static>>,
    pub wrap: Option<Wrap>,
    pub borders: Option<Borders>,
    pub style: Option<Style>,
    pub title: Option<&'static str>,
}

impl TopState {
    pub fn text(mut self, text: Text<'static>) -> Self {
        self.text = Some(text);
        self
    }

    pub fn wrap(mut self, wrap: Wrap) -> Self {
        self.wrap = Some(wrap);
        self
    }

    pub fn borders(mut self, borders: Borders) -> Self {
        self.borders = Some(borders);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    pub fn title(mut self, title: &'static str) -> Self {
        self.title = Some(title);
        self
    }
}

impl Into<Paragraph<'static>> for TopState {
    fn into(self) -> Paragraph<'static> {
        Paragraph::new(self.text.unwrap())
            .wrap(self.wrap.unwrap())
            .block(
                Block::default()
                    .style(self.style.unwrap())
                    .borders(self.borders.unwrap())
                    .title(self.title.unwrap()),
            )
    }
}
