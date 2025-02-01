use ratatui::{buffer::Buffer, layout::Rect, widgets::StatefulWidget};

#[derive(Default)]
pub struct Terminal {}

impl StatefulWidget for Terminal {
    type State = TerminalState;

    fn render(self, _area: Rect, _buf: &mut Buffer, _state: &mut Self::State) {}
}

#[derive(Default)]
pub struct TerminalState {}

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
