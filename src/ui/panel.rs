use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Widget},
};

pub fn inner_area(area: Rect) -> Rect {
    Block::bordered().inner(area)
}

pub struct Panel<'a, W> {
    content: W,
    title: Line<'a>,
    right_title: Option<Line<'a>>,
    focused: bool,
}

impl<'a, W> Panel<'a, W> {
    pub fn new<T>(content: W, title: T) -> Self
    where
        T: Into<Line<'a>>,
    {
        Self {
            content,
            title: title.into(),
            right_title: None,
            focused: false,
        }
    }

    pub fn right_title<T>(mut self, title: T) -> Self
    where
        T: Into<Line<'a>>,
    {
        self.right_title = Some(
            title
                .into()
                .alignment(Alignment::Right)
                .style(Style::default().fg(Color::Reset)),
        );
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

impl<W: Widget> Widget for Panel<'_, W> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().dim()
        };

        let mut block = Block::bordered()
            .title(self.title)
            .border_style(border_style);

        if let Some(right_title) = self.right_title {
            block = block.title(right_title);
        }

        let inner = block.inner(area);
        block.render(area, buf);

        self.content.render(inner, buf);
    }
}
