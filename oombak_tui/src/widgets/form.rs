use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Clear, Paragraph, StatefulWidget, Widget},
};

use crate::{
    styles::form::{
        HIGHLIGHTED_INPUT_FIELD_BORDER_STYLE, INPUT_FIELD_STYLE, NORMAL_FIELD_BORDER_STYLE,
    },
    utils,
};

use super::{
    confirmation_buttons::ConfirmationButtons, CommandLine, CommandLineState, ConfirmationState,
};

#[derive(Default)]
pub struct Form {}

pub struct FormState {
    input_fields: Vec<InputField>,
    highlight: Option<FormHighlight>,
}

pub struct InputField {
    name: String,
    input_state: CommandLineState,
}

enum FormHighlight {
    InputField(usize),
    Apply,
    Cancel,
}

impl StatefulWidget for Form {
    type State = FormState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = Self::get_render_area(area, state);
        let block = Block::bordered();
        let inner_area = block.inner(area);
        let areas = Layout::vertical(vec![
            Constraint::Length(Self::input_fields_height(state) as u16),
            Constraint::Length(Self::confirmation_height() as u16),
        ])
        .split(inner_area);
        let confirmation_area = areas[1];
        let input_field_areas =
            Layout::vertical(vec![Constraint::Length(3); state.input_fields.len()]).split(areas[0]);
        Clear.render(area, buf);
        block.render(area, buf);
        Self::render_input_fields(&input_field_areas, buf, state);
        Self::render_confirmation_buttons(confirmation_area, buf, state);
    }
}

impl Form {
    fn get_render_area(rect: Rect, state: &FormState) -> Rect {
        let height = Self::input_fields_height(state) + Self::confirmation_height() + 2;
        let width = usize::min(
            rect.width as usize,
            Self::input_field_name_width(state) + 50,
        );
        let height = rect.height.min(height as u16);
        let width = rect.width.min(width as u16);
        utils::layout::get_popup_area_centered(rect, width, height)
    }

    fn render_input_fields(areas: &[Rect], buf: &mut Buffer, state: &mut FormState) {
        let input_field_name_width = Self::input_field_name_width(state);
        for (i, (input_field, area)) in state.input_fields.iter_mut().zip(areas).enumerate() {
            let highlight = matches!(state.highlight, Some(FormHighlight::InputField(x)) if x == i);
            Self::render_input_field(*area, buf, input_field, input_field_name_width, highlight);
        }
    }

    fn input_field_name_width(state: &FormState) -> usize {
        Self::max_input_field_name_length(state) + 4
    }

    fn input_fields_height(state: &FormState) -> usize {
        3 * state.input_fields.len()
    }

    fn confirmation_height() -> usize {
        1
    }

    fn max_input_field_name_length(state: &FormState) -> usize {
        state
            .input_fields
            .iter()
            .map(|i| i.name.len())
            .max()
            .unwrap_or(0)
    }

    fn render_input_field(
        area: Rect,
        buf: &mut Buffer,
        input_field: &mut InputField,
        input_field_name_width: usize,
        highlight: bool,
    ) {
        let areas = Layout::horizontal(vec![
            Constraint::Length(input_field_name_width as u16),
            Constraint::Min(0),
        ])
        .split(area);
        Self::render_input_field_name(areas[0], buf, &input_field.name);
        Self::render_input_text(areas[1], buf, &mut input_field.input_state, highlight);
    }

    fn render_input_field_name(area: Rect, buf: &mut Buffer, input_field_name: &str) {
        let input_field_name = format!("{input_field_name}: ");
        let input_field_name = Paragraph::new(input_field_name);
        let block = Block::bordered();
        let inner_area = block.inner(area);
        input_field_name.render(inner_area, buf);
    }

    fn render_input_text(
        area: Rect,
        buf: &mut Buffer,
        input_state: &mut CommandLineState,
        highlight: bool,
    ) {
        let mut block = Block::bordered();
        let inner_area = block.inner(area);
        let mut input_field = CommandLine::default()
            .no_header()
            .line_style(INPUT_FIELD_STYLE);

        if highlight {
            block = block.style(HIGHLIGHTED_INPUT_FIELD_BORDER_STYLE);
        } else {
            block = block.style(NORMAL_FIELD_BORDER_STYLE);
            input_field = input_field.cursor_style(Style::new().bg(Color::Reset).fg(Color::Reset));
        };

        block.render(area, buf);
        input_field.render(inner_area, buf, input_state);
    }

    fn render_confirmation_buttons(area: Rect, buf: &mut Buffer, state: &mut FormState) {
        let mut confirmation_state = match state.highlight {
            Some(FormHighlight::Apply) => ConfirmationState::confirm(),
            Some(FormHighlight::Cancel) => ConfirmationState::dismiss(),
            Some(_) | None => ConfirmationState::default(),
        };
        ConfirmationButtons::default()
            .confirm_text("Apply")
            .dismiss_text("Cancel")
            .render(area, buf, &mut confirmation_state);
    }
}

impl InputField {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            input_state: CommandLineState::default(),
        }
    }
}

impl FormState {
    pub fn new(input_fields: Vec<InputField>) -> Self {
        let highlight = if input_fields.is_empty() {
            None
        } else {
            Some(FormHighlight::InputField(0))
        };
        Self {
            input_fields,
            highlight,
        }
    }

    pub fn up(&mut self) {
        self.highlight = match self.highlight {
            Some(FormHighlight::InputField(0)) => Some(FormHighlight::InputField(0)),
            Some(FormHighlight::InputField(x)) => Some(FormHighlight::InputField(x - 1)),
            Some(FormHighlight::Apply) | Some(FormHighlight::Cancel) => {
                Some(FormHighlight::InputField(self.input_fields.len() - 1))
            }
            None => None,
        }
    }

    pub fn down(&mut self) {
        self.highlight = match self.highlight {
            Some(FormHighlight::InputField(x)) if x == self.input_fields.len() - 1 => {
                Some(FormHighlight::Apply)
            }
            Some(FormHighlight::InputField(x)) => Some(FormHighlight::InputField(x + 1)),
            Some(FormHighlight::Apply) => Some(FormHighlight::Apply),
            Some(FormHighlight::Cancel) => Some(FormHighlight::Cancel),
            None => None,
        }
    }

    pub fn left(&mut self) {
        match self.highlight {
            Some(FormHighlight::Apply) | Some(FormHighlight::Cancel) => {
                self.highlight = Some(FormHighlight::Apply)
            }
            Some(FormHighlight::InputField(x)) => {
                self.input_fields[x].input_state.move_cursor_left()
            }
            None => (),
        }
    }

    pub fn right(&mut self) {
        match self.highlight {
            Some(FormHighlight::Apply) | Some(FormHighlight::Cancel) => {
                self.highlight = Some(FormHighlight::Cancel)
            }
            Some(FormHighlight::InputField(x)) => {
                self.input_fields[x].input_state.move_cursor_right()
            }
            None => (),
        }
    }

    pub fn put(&mut self, ch: char) {
        if let Some(FormHighlight::InputField(x)) = self.highlight {
            self.input_fields[x].input_state.put(ch)
        }
    }

    pub fn backspace(&mut self) {
        if let Some(FormHighlight::InputField(x)) = self.highlight {
            self.input_fields[x].input_state.backspace()
        }
    }
}
