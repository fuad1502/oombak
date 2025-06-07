use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};

use super::confirmation_buttons::{ConfirmationButtons, ConfirmationState};

pub struct ConfirmationBox<'a> {
    text: &'a Paragraph<'a>,
}

impl<'a> ConfirmationBox<'a> {
    pub fn new(text: &'a Paragraph<'a>) -> Self {
        Self { text }
    }
}

impl StatefulWidget for ConfirmationBox<'_> {
    type State = ConfirmationState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::bordered();
        let inner = block.inner(area);

        let chunks = Layout::vertical(vec![Constraint::Min(0), Constraint::Length(1)]).split(inner);
        let text_area = chunks[0];

        block.render(area, buf);
        self.text.render(text_area, buf);
        ConfirmationButtons::default().render(chunks[1], buf, state);
    }
}
