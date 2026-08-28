use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    text::Line,
    widgets::Block,
};

pub struct BlockBuilder<'a> {
    block: Block<'a>,
}

impl<'a> BlockBuilder<'a> {
    pub fn new() -> Self {
        BlockBuilder {
            block: Block::bordered(),
        }
    }

    pub fn title<T>(mut self, title: T) -> Self
    where
        T: Into<Line<'a>>,
    {
        self.block = self.block.title(title);
        self
    }

    pub fn right_title<T>(mut self, title: T) -> Self
    where
        T: Into<Line<'a>>,
    {
        let right_title = title.into().alignment(Alignment::Right);

        self.block = self.block.title(right_title);

        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        let style = if focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().dim()
        };

        self.block = self.block.border_style(style);
        self
    }

    pub fn build(self) -> Block<'a> {
        self.block
    }
}

impl<'a> Default for BlockBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}
