use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Span,
    widgets::{StatefulWidget, Widget},
};

use crate::styles::wave_viewer::{CURSOR_STYLE, TIMEBAR_STYLE, TIME_INDICATOR_STYLE};

use super::ScrollState;

#[derive(Default)]
pub struct TimeBar {
    tick_count: usize,
    tick_period: f64,
    time_unit: TimeUnit,
}

#[derive(Default)]
pub enum TimeUnit {
    #[default]
    Picoseconds,
}

impl TimeBar {
    pub fn tick_count(mut self, tick_count: usize) -> Self {
        self.tick_count = tick_count;
        self
    }

    pub fn tick_period(mut self, tick_period: f64) -> Self {
        self.tick_period = tick_period;
        self
    }
}

impl StatefulWidget for TimeBar {
    type State = ScrollState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.set_viewport_length(area.width as usize);
        if area.width >= 1 && area.height >= 3 {
            self.render_time_indicator(area, buf, state);
            let lines = self.plot_into_lines(state);
            buf.set_string(area.x, area.y + 1, &lines[0], TIMEBAR_STYLE);
            buf.set_string(area.x, area.y + 2, &lines[1], TIMEBAR_STYLE);
            Self::set_highlight(buf, area, state, CURSOR_STYLE);
        }
    }
}

impl TimeBar {
    fn render_time_indicator(&self, area: Rect, buf: &mut Buffer, state: &ScrollState) {
        let time_indicator_area = Rect::new(area.x, area.y, area.width, 1);
        let span = Span::from(format!(" {:10} ", self.format(self.current_time(state))))
            .style(TIME_INDICATOR_STYLE);
        span.render(time_indicator_area, buf);
    }

    fn plot_into_lines(&self, state: &ScrollState) -> [String; 2] {
        let number_of_ticks = state.viewport_length() / self.tick_count + 2;
        let floored_start_position = self.floored_start_position(state);
        let mut lines = [String::new(), String::new()];

        let start_time = self.time_from_position(floored_start_position);
        for i in 0..number_of_ticks {
            let time = start_time + i as f64 * self.tick_period;
            lines[0] += &self.new_tick_segment_upper(time);
            lines[1] += &self.new_tick_segment_lower();
        }

        self.crop_lines(&mut lines, state);
        lines
    }

    fn set_highlight(buf: &mut Buffer, area: Rect, state: &ScrollState, highlight_style: Style) {
        if area.width >= 1 && area.height >= 2 && state.viewport_length() >= 1 {
            buf.set_style(
                Rect::new(area.x + state.selected_position() as u16, area.y + 1, 1, 2),
                highlight_style,
            );
        }
    }

    fn new_tick_segment_upper(&self, time: f64) -> String {
        format!("╻{0:1$}", self.format(time), self.tick_count - 1)
    }

    fn new_tick_segment_lower(&self) -> String {
        let sub_ticks_left = usize::saturating_sub(self.tick_count, 2) / 2;
        let sub_ticks_right = sub_ticks_left;
        let middle_ticks = self.tick_count - sub_ticks_left - sub_ticks_right - 1;
        format!(
            "┻{0:┷<1$}{0:┻<2$}{0:┷<3$}",
            "", sub_ticks_left, middle_ticks, sub_ticks_right
        )
    }

    fn crop_lines(&self, lines: &mut [String; 2], state: &ScrollState) {
        let floored_start_position = self.floored_start_position(state);
        let start_offset = state.start_position() - floored_start_position;
        lines[0] = Self::crop_line(&lines[0], start_offset, state.viewport_length());
        lines[1] = Self::crop_line(&lines[1], start_offset, state.viewport_length());
    }

    fn crop_line(line: &str, start_offset: usize, width: usize) -> String {
        line.chars().skip(start_offset).take(width).collect()
    }

    fn floored_start_position(&self, state: &ScrollState) -> usize {
        state.start_position() / self.tick_count * self.tick_count
    }

    fn current_time(&self, state: &ScrollState) -> f64 {
        self.time_from_position(state.start_position() + state.selected_position())
    }

    fn time_from_position(&self, position: usize) -> f64 {
        position as f64 * (self.tick_period / self.tick_count as f64)
    }

    fn format(&self, time: f64) -> String {
        format!("{:3.2} {}", time, self.time_unit)
    }
}

impl std::fmt::Display for TimeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeUnit::Picoseconds => f.write_str("ps"),
        }
    }
}

#[cfg(test)]
mod test {
    use ratatui::{buffer::Buffer, layout::Rect, widgets::StatefulWidget};

    use crate::styles::wave_viewer::{CURSOR_STYLE, TIMEBAR_STYLE, TIME_INDICATOR_STYLE};

    use super::{ScrollState, TimeBar};

    const X0: u16 = 10;
    const Y0: u16 = 10;

    #[test]
    pub fn test_render() {
        let (time_bar, mut state, mut buf, area) = setup(50);

        time_bar.render(buf.area, &mut buf, &mut state);

        let mut expected = Buffer::with_lines(vec![
            " 0.00 ps                                          ",
            "╻0.00 ps  ╻10.00 ps ╻20.00 ps ╻30.00 ps ╻40.00 ps ",
            "┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷",
        ]);
        expected.area = area;
        expected.set_style(Rect::new(X0, Y0, 12, 1), TIME_INDICATOR_STYLE);
        expected.set_style(Rect::new(X0, Y0 + 1, 1, 2), CURSOR_STYLE);
        expected.set_style(Rect::new(X0, Y0 + 1, area.width, 2), TIMEBAR_STYLE);

        assert_eq!(buf, expected);
    }

    #[test]
    pub fn test_scroll_little() {
        let (time_bar, mut state, mut buf, area) = setup(50);

        for _ in 0..10 {
            state.next();
        }

        time_bar.render(buf.area, &mut buf, &mut state);

        let mut expected = Buffer::with_lines(vec![
            " 10.00 ps                                         ",
            "╻0.00 ps  ╻10.00 ps ╻20.00 ps ╻30.00 ps ╻40.00 ps ",
            "┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷",
        ]);
        expected.area = area;
        expected.set_style(Rect::new(X0, Y0, 12, 1), TIME_INDICATOR_STYLE);
        expected.set_style(Rect::new(X0 + 10, Y0 + 1, 1, 2), CURSOR_STYLE);
        expected.set_style(Rect::new(X0, Y0 + 1, area.width, 2), TIMEBAR_STYLE);

        assert_eq!(buf, expected);
    }

    #[test]
    pub fn test_scroll_pass_end() {
        let (time_bar, mut state, mut buf, area) = setup(50);

        for _ in 0..55 {
            state.next();
        }

        time_bar.render(buf.area, &mut buf, &mut state);

        let mut expected = Buffer::with_lines(vec![
            " 55.00 ps                                         ",
            "ps  ╻10.00 ps ╻20.00 ps ╻30.00 ps ╻40.00 ps ╻50.00",
            "┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻",
        ]);
        expected.area = area;
        expected.set_style(Rect::new(X0, Y0, 12, 1), TIME_INDICATOR_STYLE);
        expected.set_style(Rect::new(X0 + 49, Y0 + 1, 1, 2), CURSOR_STYLE);
        expected.set_style(Rect::new(X0, Y0 + 1, area.width, 2), TIMEBAR_STYLE);

        assert_eq!(buf, expected);
    }

    #[test]
    pub fn test_scroll_pass_end_and_back() {
        let (time_bar, mut state, mut buf, area) = setup(50);

        for _ in 0..55 {
            state.next();
        }
        for _ in 0..5 {
            state.prev();
        }

        time_bar.render(buf.area, &mut buf, &mut state);

        let mut expected = Buffer::with_lines(vec![
            " 50.00 ps                                         ",
            "ps  ╻10.00 ps ╻20.00 ps ╻30.00 ps ╻40.00 ps ╻50.00",
            "┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻",
        ]);
        expected.area = area;
        expected.set_style(Rect::new(X0, Y0, 12, 1), TIME_INDICATOR_STYLE);
        expected.set_style(Rect::new(X0 + 44, Y0 + 1, 1, 2), CURSOR_STYLE);
        expected.set_style(Rect::new(X0, Y0 + 1, area.width, 2), TIMEBAR_STYLE);

        assert_eq!(buf, expected);
    }

    #[test]
    pub fn test_scroll_pass_end_and_pass_back() {
        let (time_bar, mut state, mut buf, area) = setup(50);

        for _ in 0..55 {
            state.next();
        }
        for _ in 0..50 {
            state.prev();
        }

        time_bar.render(buf.area, &mut buf, &mut state);

        let mut expected = Buffer::with_lines(vec![
            " 5.00 ps                                          ",
            " ps  ╻10.00 ps ╻20.00 ps ╻30.00 ps ╻40.00 ps ╻50.0",
            "┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷",
        ]);
        expected.area = area;
        expected.set_style(Rect::new(X0, Y0, 12, 1), TIME_INDICATOR_STYLE);
        expected.set_style(Rect::new(X0, Y0 + 1, 1, 2), CURSOR_STYLE);
        expected.set_style(Rect::new(X0, Y0 + 1, area.width, 2), TIMEBAR_STYLE);

        assert_eq!(buf, expected);
    }

    #[test]
    pub fn test_scroll_pass_end_uneven_viewport() {
        let (time_bar, mut state, mut buf, area) = setup(49);

        for _ in 0..55 {
            state.next();
        }

        time_bar.render(buf.area, &mut buf, &mut state);

        let mut expected = Buffer::with_lines(vec![
            " 55.00 ps                                        ",
            "s  ╻10.00 ps ╻20.00 ps ╻30.00 ps ╻40.00 ps ╻50.00",
            "┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻",
        ]);
        expected.area = area;
        expected.set_style(Rect::new(X0, Y0, 12, 1), TIME_INDICATOR_STYLE);
        expected.set_style(Rect::new(X0 + 48, Y0 + 1, 1, 2), CURSOR_STYLE);
        expected.set_style(Rect::new(X0, Y0 + 1, area.width, 2), TIMEBAR_STYLE);

        assert_eq!(buf, expected);
    }

    fn setup(viewport_length: usize) -> (TimeBar, ScrollState, Buffer, Rect) {
        let time_bar = TimeBar::default().tick_period(10.0).tick_count(10);
        let mut state = ScrollState::default();
        state.set_content_length(100);

        let area = Rect::new(X0, Y0, viewport_length as u16, 3);
        let buf = Buffer::empty(area);
        state.set_viewport_length(viewport_length);

        (time_bar, state, buf, area)
    }
}
