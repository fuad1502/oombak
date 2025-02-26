use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Styled,
    widgets::{List, ListDirection, ListItem, ListState, StatefulWidget},
};

use crate::styles::terminal::{FAIL_OUTPUT_STYLE, SUCCESS_OUTPUT_STYLE};

use super::{CommandLine, CommandLineState};

#[derive(Default)]
pub struct Terminal {}

#[derive(Default)]
pub struct TerminalState {
    command_line_state: CommandLineState,
    output_history: Vec<Result<String, String>>,
    history_list_state: ListState,
}

impl StatefulWidget for Terminal {
    type State = TerminalState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let chunks = Layout::vertical(vec![Constraint::Min(0), Constraint::Length(1)]).split(area);
        Self::render_output_history(chunks[0], buf, state);
        CommandLine::default().render(chunks[1], buf, &mut state.command_line_state);
    }
}

impl Terminal {
    fn render_output_history(area: Rect, buf: &mut Buffer, state: &mut TerminalState) {
        let list_items = Self::new_list_items_from_output_history(&state.output_history);
        let list = List::new(list_items).direction(ListDirection::BottomToTop);
        StatefulWidget::render(list, area, buf, &mut state.history_list_state);
    }

    fn new_list_items_from_output_history<'a>(
        output_history: &[Result<String, String>],
    ) -> Vec<ListItem<'a>> {
        output_history
            .iter()
            .rev()
            .map(|h| match h {
                Ok(output) => ListItem::from(format!("> {output}")).set_style(SUCCESS_OUTPUT_STYLE),
                Err(output) => ListItem::from(format!("> {output}")).set_style(FAIL_OUTPUT_STYLE),
            })
            .collect()
    }
}

impl TerminalState {
    pub fn command_line_state(&self) -> &CommandLineState {
        &self.command_line_state
    }

    pub fn command_line_state_mut(&mut self) -> &mut CommandLineState {
        &mut self.command_line_state
    }

    pub fn output_history(&self) -> &Vec<Result<String, String>> {
        &self.output_history
    }

    pub fn append_output_history(&mut self, output: Result<String, String>) {
        self.output_history.push(output);
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
