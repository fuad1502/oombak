use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::{StatefulWidget, Widget},
};

use crate::styles::terminal::{COMMAND_LINE_HEADER_STYLE, COMMAND_LINE_STYLE, TEXT_CURSOR_STYLE};

use super::ScrollState;

pub struct CommandLine {
    no_header: bool,
    line_style: Style,
    cursor_style: Style,
}

#[derive(Default)]
pub struct CommandLineState {
    text: String,
    cursor_position: usize,
    scroll_state: ScrollState,
}

impl StatefulWidget for CommandLine {
    type State = CommandLineState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let input_width = if self.no_header {
            area.width as usize
        } else {
            (area.width as usize).saturating_sub(Self::HEADER.len() + 1)
        };
        state
            .scroll_state
            .set_viewport_length(input_width.saturating_sub(1));

        let start_idx = state.scroll_state.start_position();
        let highlight_idx =
            get_utf8_index(&state.text, state.cursor_position).unwrap_or(state.text.len());

        let highlight = state.text.get(highlight_idx..=highlight_idx).unwrap_or(" ");
        let before_highlight = state.text.get(start_idx..highlight_idx).unwrap_or(" ");
        let after_highlight = state.text.get(highlight_idx + 1..).unwrap_or(" ");

        let mut command_line_components = vec![
            Span::from(Self::HEADER).style(COMMAND_LINE_HEADER_STYLE),
            Span::from(" "),
            Span::from(before_highlight),
            Span::from(highlight).style(self.cursor_style),
            Span::from(after_highlight),
        ];
        if self.no_header {
            command_line_components.remove(0);
            command_line_components.remove(0);
        }

        let command_line = Line::from(command_line_components).style(self.line_style);

        command_line.render(area, buf);
    }
}

impl Default for CommandLine {
    fn default() -> Self {
        Self {
            no_header: false,
            line_style: COMMAND_LINE_STYLE,
            cursor_style: TEXT_CURSOR_STYLE,
        }
    }
}

impl CommandLine {
    const HEADER: &'static str = " >>> ";

    pub fn no_header(mut self) -> Self {
        self.no_header = true;
        self
    }

    pub fn line_style(mut self, style: Style) -> Self {
        self.line_style = style;
        self
    }

    pub fn cursor_style(mut self, style: Style) -> Self {
        self.cursor_style = style;
        self
    }
}

impl CommandLineState {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_position = 0;
        self.scroll_state.set_content_length(0);
    }

    pub fn put(&mut self, ch: char) {
        let idx = get_utf8_index(&self.text, self.cursor_position).unwrap_or(self.text.len());
        self.text.insert(idx, ch);
        self.cursor_position += 1;
        self.scroll_state.set_content_length(self.text.len());
        self.scroll_state.next();
    }

    pub fn backspace(&mut self) {
        if self.cursor_position >= 1 {
            let idx = get_utf8_index(&self.text, self.cursor_position - 1).unwrap();
            self.text.remove(idx);
            self.cursor_position = self.cursor_position.saturating_sub(1);
            self.scroll_state.set_content_length(self.text.len());
            self.scroll_state.prev();
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.text.len() {
            self.cursor_position += 1;
        }
        self.scroll_state.next();
    }

    pub fn move_cursor_left(&mut self) {
        self.cursor_position = self.cursor_position.saturating_sub(1);
        self.scroll_state.prev();
    }
}

fn get_utf8_index(s: &str, ch_idx: usize) -> Option<usize> {
    s.char_indices().nth(ch_idx).map(|(i, _)| i)
}
