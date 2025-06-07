use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::Text,
    widgets::{StatefulWidget, Widget},
};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct ConfirmationState {
    confirm: Option<bool>,
}

pub struct ConfirmationButtons {
    confirm_text: String,
    dismiss_text: String,
}

impl StatefulWidget for ConfirmationButtons {
    type State = ConfirmationState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let chunks = Layout::horizontal(vec![
            Constraint::Min(0),
            Constraint::Length(8),
            Constraint::Min(0),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(area);
        let confirm_area = chunks[1];
        let dismiss_area = chunks[3];

        let selected_style = Style::new().white().on_red();
        let not_selected_style = Style::new().black().on_white();

        let mut confirm_text = Text::from(self.confirm_text)
            .centered()
            .style(not_selected_style);
        let mut dismiss_text = Text::from(self.dismiss_text)
            .centered()
            .style(not_selected_style);

        match state.is_confirm() {
            Some(true) => confirm_text = confirm_text.style(selected_style),
            Some(false) => dismiss_text = dismiss_text.style(selected_style),
            None => (),
        }

        confirm_text.render(confirm_area, buf);
        dismiss_text.render(dismiss_area, buf);
    }
}

impl Default for ConfirmationButtons {
    fn default() -> Self {
        Self {
            confirm_text: String::from("Yes"),
            dismiss_text: String::from("No"),
        }
    }
}

impl ConfirmationButtons {
    pub fn confirm_text(mut self, text: &str) -> Self {
        self.confirm_text = String::from(text);
        self
    }

    pub fn dismiss_text(mut self, text: &str) -> Self {
        self.dismiss_text = String::from(text);
        self
    }
}

impl ConfirmationState {
    pub fn confirm() -> Self {
        ConfirmationState {
            confirm: Some(true),
        }
    }

    pub fn dismiss() -> Self {
        ConfirmationState {
            confirm: Some(false),
        }
    }

    pub fn set_confirm(&mut self) {
        self.confirm = Some(true);
    }

    pub fn set_dismiss(&mut self) {
        self.confirm = Some(false);
    }

    pub fn is_confirm(&self) -> Option<bool> {
        self.confirm
    }
}
