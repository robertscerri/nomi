use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{StatefulWidget, Widget},
};

pub struct Stateful<W>
where
    W: StatefulWidget,
    W::State: Sized,
{
    widget: W,
    state: W::State,
}

impl<W> Stateful<W>
where
    W: StatefulWidget,
    W::State: Sized,
{
    pub fn new(widget: W, state: W::State) -> Self {
        Self { widget, state }
    }
}

impl<W> Widget for Stateful<W>
where
    W: StatefulWidget,
    W::State: Sized,
{
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        self.widget.render(area, buf, &mut self.state);
    }
}
