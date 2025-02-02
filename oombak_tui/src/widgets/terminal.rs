use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::{Line, Span},
    widgets::{List, ListDirection, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

use super::ScrollState;

#[derive(Default)]
pub struct Terminal {}

#[derive(Default)]
pub struct TerminalState {
    command_line_state: CommandLineState,
    output_history: Vec<Result<String, String>>,
    history_list_state: ListState,
}

#[derive(Default)]
pub struct CommandLineState {
    text: String,
    cursor_position: usize,
    scroll_state: ScrollState,
}

impl StatefulWidget for Terminal {
    type State = TerminalState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let chunks = Layout::vertical(vec![Constraint::Min(0), Constraint::Length(1)]).split(area);
        let list_items = Self::new_list_items_from_output_history(&state.output_history);
        let list = List::new(list_items).direction(ListDirection::BottomToTop);
        StatefulWidget::render(list, chunks[0], buf, &mut state.history_list_state);

        let state = &mut state.command_line_state;
        state
            .scroll_state
            .set_viewport_length(area.width as usize - 7);
        let start_idx = state.scroll_state.start_position();
        let highlight_idx = state
            .text
            .char_indices()
            .nth(state.cursor_position)
            .unwrap_or((state.text.len(), ' '))
            .0;
        let command_line = Line::from(vec![
            Span::from(" >>> ").black().on_yellow(),
            Span::from(" "),
            Span::from(state.text.get(start_idx..highlight_idx).unwrap_or(" ")),
            Span::from(state.text.get(highlight_idx..=highlight_idx).unwrap_or(" "))
                .black()
                .on_white(),
            Span::from(state.text.get(highlight_idx + 1..).unwrap_or(" ")),
        ]);
        let paragraph = Paragraph::new(command_line).on_blue();
        paragraph.render(chunks[1], buf);
    }
}

impl TerminalState {
    pub fn command_line(&self) -> &str {
        &self.command_line_state.text
    }

    pub fn output_history(&self) -> &Vec<Result<String, String>> {
        &self.output_history
    }

    pub fn clear_command_line(&mut self) {
        self.command_line_state.text.clear();
        self.command_line_state.cursor_position = 0;
        self.command_line_state.scroll_state.set_content_length(0);
    }

    pub fn put(&mut self, c: char) {
        let state = &mut self.command_line_state;
        let idx = state
            .text
            .char_indices()
            .nth(state.cursor_position)
            .unwrap_or((state.text.len(), ' '))
            .0;
        state.text.insert(idx, c);
        state.cursor_position += 1;
        state.scroll_state.set_content_length(state.text.len());
        state.scroll_state.next();
    }

    pub fn backspace(&mut self) {
        let state = &mut self.command_line_state;
        if state.cursor_position >= 1 {
            let idx = state
                .text
                .char_indices()
                .nth(state.cursor_position - 1)
                .unwrap()
                .0;
            state.text.remove(idx);
            state.cursor_position = state.cursor_position.saturating_sub(1);
            state.scroll_state.set_content_length(state.text.len());
            state.scroll_state.prev();
        }
    }

    pub fn move_cursor_right(&mut self) {
        let state = &mut self.command_line_state;
        if state.cursor_position < state.text.len() {
            state.cursor_position += 1;
        }
        state.scroll_state.next();
    }

    pub fn move_cursor_left(&mut self) {
        let state = &mut self.command_line_state;
        state.cursor_position = state.cursor_position.saturating_sub(1);
        state.scroll_state.prev();
    }

    pub fn append_output_history(&mut self, output: Result<String, String>) {
        self.output_history.push(output);
    }
}

impl Terminal {
    fn new_list_items_from_output_history<'a>(
        output_history: &[Result<String, String>],
    ) -> Vec<ListItem<'a>> {
        output_history
            .iter()
            .rev()
            .map(|h| match h {
                Ok(output) => ListItem::from(format!("> {output}")).green(),
                Err(output) => ListItem::from(format!("> {output}")).red(),
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use ratatui::{buffer::Buffer, layout::Rect};

    const X0: u16 = 10;
    const Y0: u16 = 10;

    #[test]
    fn test_render() {
        let area = Rect::new(X0, Y0, 50 as u16, 10);
        let buf = Buffer::empty(area);

        //render

        let mut expected = Buffer::with_lines(vec!["", "", "", "", "", "", "", "", "", ""]);
        expected.area = area;

        assert_eq!(buf, expected);
    }
}
