use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Clear, Paragraph, StatefulWidget, Widget},
};

use crate::{
    styles::form::{
        HIGHLIGHTED_INPUT_FIELD_BORDER_STYLE, INPUT_FIELD_STYLE, NORMAL_FIELD_BORDER_STYLE,
    },
    utils,
    widgets::{DropDown, DropDownState},
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
    input_type: InputFieldType,
}

pub enum InputFieldType {
    CommandLine(CommandLineState),
    DropDown(DropDownState),
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
        let block = Block::bordered().border_type(BorderType::Rounded);
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
        Self::render_confirmation_buttons(confirmation_area, buf, state);
        // Opened dropdown needs to be rendered last because it draws beyond its borders
        Self::render_input_fields(&input_field_areas, buf, state);
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
        let mut opened_dropdown_idx = None;

        for (i, (input_field, area)) in state.input_fields.iter_mut().zip(areas).enumerate() {
            // Opened dropdown needs to be rendered last because it draws beyond its borders
            if let InputFieldType::DropDown(state) = &input_field.input_type {
                if state.is_opened() {
                    opened_dropdown_idx = Some(i);
                    continue;
                }
            }

            let highlight = matches!(state.highlight, Some(FormHighlight::InputField(x)) if x == i);
            Self::render_input_field(*area, buf, input_field, input_field_name_width, highlight);
        }

        // Opened dropdown needs to be rendered last because it draws beyond its borders
        if let Some(idx) = opened_dropdown_idx {
            Self::render_input_field(
                areas[idx],
                buf,
                &mut state.input_fields[idx],
                input_field_name_width,
                true,
            );
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
        match &mut input_field.input_type {
            InputFieldType::CommandLine(state) => {
                Self::render_input_text(areas[1], buf, state, highlight)
            }
            InputFieldType::DropDown(state) => {
                Self::render_dropdown(areas[1], buf, state, highlight)
            }
        }
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
        command_line_state: &mut CommandLineState,
        highlight: bool,
    ) {
        let mut block = Block::bordered().border_type(BorderType::Rounded);
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
        input_field.render(inner_area, buf, command_line_state);
    }

    fn render_dropdown(
        area: Rect,
        buf: &mut Buffer,
        drop_down_state: &mut DropDownState,
        highlight: bool,
    ) {
        let mut block = Block::bordered().border_type(BorderType::Rounded);
        if highlight {
            block = block.style(HIGHLIGHTED_INPUT_FIELD_BORDER_STYLE);
        } else {
            block = block.style(NORMAL_FIELD_BORDER_STYLE);
        };
        let dropdown = DropDown::default().block(block);
        dropdown.render(area, buf, drop_down_state);
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
    pub fn text(name: &str) -> Self {
        Self {
            name: name.to_string(),
            input_type: InputFieldType::CommandLine(CommandLineState::default()),
        }
    }

    pub fn dropdown(name: &str, items: &[&str]) -> Self {
        let items = items.iter().map(|x| String::from(*x)).collect();
        Self {
            name: name.to_string(),
            input_type: InputFieldType::DropDown(DropDownState::new(items)),
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
        if let Some(state) = self.try_get_dropdown_state_from_selected() {
            if state.is_opened() {
                state.up();
                return;
            }
        }
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
        if let Some(state) = self.try_get_dropdown_state_from_selected() {
            if state.is_opened() {
                state.down();
                return;
            }
        }
        self.highlight = match self.highlight {
            Some(FormHighlight::InputField(x)) if x == self.input_fields.len() - 1 => {
                Some(FormHighlight::Apply)
            }
            Some(FormHighlight::InputField(x)) => Some(FormHighlight::InputField(x + 1)),
            Some(FormHighlight::Apply) => Some(FormHighlight::Cancel),
            Some(FormHighlight::Cancel) => Some(FormHighlight::Cancel),
            None => None,
        }
    }

    pub fn left(&mut self) {
        if let Some(state) = self.try_get_command_line_state_from_selected() {
            state.move_cursor_left();
            return;
        }
        match self.highlight {
            Some(FormHighlight::Apply) | Some(FormHighlight::Cancel) => {
                self.highlight = Some(FormHighlight::Apply)
            }
            _ => (),
        }
    }

    pub fn right(&mut self) {
        if let Some(state) = self.try_get_command_line_state_from_selected() {
            state.move_cursor_right();
            return;
        }
        match self.highlight {
            Some(FormHighlight::Apply) | Some(FormHighlight::Cancel) => {
                self.highlight = Some(FormHighlight::Cancel)
            }
            _ => (),
        }
    }

    pub fn put(&mut self, ch: char) {
        if let Some(state) = self.try_get_command_line_state_from_selected() {
            state.put(ch);
        }
    }

    pub fn backspace(&mut self) {
        if let Some(state) = self.try_get_command_line_state_from_selected() {
            state.backspace();
        }
    }

    pub fn enter(&mut self) {
        if let Some(state) = self.try_get_dropdown_state_from_selected() {
            if state.is_opened() {
                state.close()
            } else {
                state.open()
            }
        }
    }

    pub fn is_apply(&self) -> bool {
        matches!(self.highlight, Some(FormHighlight::Apply))
    }

    pub fn is_cancel(&self) -> bool {
        matches!(self.highlight, Some(FormHighlight::Cancel))
    }

    pub fn is_dropdown(&self) -> bool {
        if let Some(FormHighlight::InputField(x)) = self.highlight {
            if let InputFieldType::DropDown(_) = &self.input_fields[x].input_type {
                return true;
            }
        }
        false
    }

    pub fn is_command_line(&self) -> bool {
        if let Some(FormHighlight::InputField(x)) = self.highlight {
            if let InputFieldType::CommandLine(_) = &self.input_fields[x].input_type {
                return true;
            }
        }
        false
    }

    pub fn entries(&self) -> Vec<String> {
        self.input_fields
            .iter()
            .map(|i| match &i.input_type {
                InputFieldType::CommandLine(state) => state.text().to_string(),
                InputFieldType::DropDown(state) => state.selected().unwrap_or_default().to_string(),
            })
            .collect()
    }

    fn try_get_dropdown_state_from_selected(&mut self) -> Option<&mut DropDownState> {
        if let Some(FormHighlight::InputField(x)) = self.highlight {
            if let InputFieldType::DropDown(state) = &mut self.input_fields[x].input_type {
                return Some(state);
            }
        }
        None
    }

    fn try_get_command_line_state_from_selected(&mut self) -> Option<&mut CommandLineState> {
        if let Some(FormHighlight::InputField(x)) = self.highlight {
            if let InputFieldType::CommandLine(state) = &mut self.input_fields[x].input_type {
                return Some(state);
            }
        }
        None
    }
}
