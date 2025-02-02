use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::{Line, Span},
    widgets::{List, ListDirection, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

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
}

impl StatefulWidget for Terminal {
    type State = TerminalState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let chunks = Layout::vertical(vec![Constraint::Min(0), Constraint::Length(1)]).split(area);
        let list_items = Self::new_list_items_from_output_history(&state.output_history);
        let list = List::new(list_items).direction(ListDirection::BottomToTop);
        StatefulWidget::render(list, chunks[0], buf, &mut state.history_list_state);

        let command_line = Line::from(vec![
            Span::from(" >>> ").black().on_yellow(),
            Span::from(" "),
            Span::from(&state.command_line_state.text),
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
        self.command_line_state.text = "".to_string();
        self.command_line_state.cursor_position = 0;
    }

    pub fn put(&mut self, c: char) {
        self.command_line_state.text += &c.to_string();
    }

    pub fn backspace(&mut self) {
        self.command_line_state.text.pop();
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
