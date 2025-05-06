use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::Text,
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};

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

        let chunks = Layout::horizontal(vec![
            Constraint::Min(0),
            Constraint::Length(8),
            Constraint::Min(0),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(chunks[1]);
        let confirm_area = chunks[1];
        let dismiss_area = chunks[3];

        let selected_style = Style::new().white().on_red();
        let not_selected_style = Style::new().black().on_white();

        let mut confirm_text = Text::from("Yes").centered().style(not_selected_style);
        let mut dismiss_text = Text::from("No").centered().style(not_selected_style);

        if state.confirm {
            confirm_text = confirm_text.style(selected_style);
        } else {
            dismiss_text = dismiss_text.style(selected_style);
        }

        block.render(area, buf);
        self.text.render(text_area, buf);

        confirm_text.render(confirm_area, buf);
        dismiss_text.render(dismiss_area, buf);
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct ConfirmationState {
    confirm: bool,
}

impl ConfirmationState {
    pub fn confirm() -> Self {
        ConfirmationState { confirm: true }
    }

    pub fn dismiss() -> Self {
        ConfirmationState { confirm: false }
    }

    pub fn set_confirm(&mut self) {
        self.confirm = true;
    }

    pub fn set_dismiss(&mut self) {
        self.confirm = false;
    }

    pub fn is_confirm(&self) -> bool {
        self.confirm
    }
}
