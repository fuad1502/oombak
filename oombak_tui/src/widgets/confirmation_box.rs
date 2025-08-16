use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Paragraph, StatefulWidget, Widget},
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
        let chunks = Layout::vertical(vec![
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);
        let text_area = chunks[0];
        self.text.render(text_area, buf);
        ConfirmationButtons::default().render(chunks[2], buf, state);
    }
}
