use ratatui::{
    buffer::Buffer, layout::Rect, style::Style, style::Stylize, widgets::StatefulWidget,
};

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
    Nanoseconds,
    Miliseconds,
}

impl TimeBar {
    pub fn time_unit(mut self, time_unit: TimeUnit) -> Self {
        self.time_unit = time_unit;
        self
    }

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
    type State = TimeBarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.set_viewport_length(area.width as usize);
        if area.width >= 1 && area.height >= 2 && state.viewport_length >= 1 {
            let lines = self.plot_into_lines(state);
            buf.set_string(area.x, area.y, &lines[0], Style::default());
            buf.set_string(area.x, area.y + 1, &lines[1], Style::default());
            Self::set_highlight(buf, area, state, Style::default().on_red());
        }
    }
}

impl TimeBar {
    fn plot_into_lines(&self, state: &TimeBarState) -> [String; 2] {
        let number_of_ticks = state.viewport_length / self.tick_count + 2;
        let floored_start_position = self.floored_start_position(state);
        let mut lines = [String::new(), String::new()];

        let start_time =
            floored_start_position as f64 * (self.tick_period / self.tick_count as f64);
        for i in 0..number_of_ticks {
            let time = start_time + i as f64 * self.tick_period;
            lines[0] += &self.new_tick_segment_upper(time);
            lines[1] += &self.new_tick_segment_lower();
        }

        self.crop_lines(&mut lines, state);
        lines
    }

    fn set_highlight(buf: &mut Buffer, area: Rect, state: &TimeBarState, highlight_style: Style) {
        if area.width >= 1 && area.height >= 2 && state.viewport_length >= 1 {
            buf.set_style(
                Rect::new(area.x + state.selected_position as u16, area.y, 1, 2),
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

    fn crop_lines(&self, lines: &mut [String; 2], state: &TimeBarState) {
        let floored_start_position = self.floored_start_position(state);
        let start_offset = state.start_position - floored_start_position;
        lines[0] = Self::crop_line(&lines[0], start_offset, state.viewport_length);
        lines[1] = Self::crop_line(&lines[1], start_offset, state.viewport_length);
    }

    fn crop_line(line: &str, start_offset: usize, width: usize) -> String {
        line.chars().skip(start_offset).take(width).collect()
    }

    fn floored_start_position(&self, state: &TimeBarState) -> usize {
        state.start_position / self.tick_count * self.tick_count
    }

    fn format(&self, time: f64) -> String {
        format!("{} {}", time, self.time_unit)
    }
}

impl std::fmt::Display for TimeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeUnit::Picoseconds => f.write_str("ps"),
            TimeUnit::Nanoseconds => f.write_str("ns"),
            TimeUnit::Miliseconds => f.write_str("ms"),
        }
    }
}

#[derive(Default)]
pub struct TimeBarState {
    content_length: usize,
    start_position: usize,
    selected_position: usize,
    viewport_length: usize,
}

impl TimeBarState {
    pub fn new(total_time: usize) -> Self {
        Self {
            content_length: total_time,
            ..Default::default()
        }
    }

    pub fn set_content_length(&mut self, total_time: usize) {
        self.content_length = total_time;
    }

    pub fn set_viewport_length(&mut self, viewport_length: usize) {
        let viewport_length = usize::min(viewport_length, self.content_length);
        if self.selected_position >= viewport_length {
            self.selected_position = usize::saturating_sub(self.viewport_length, 1);
        }
        self.viewport_length = viewport_length;
    }

    pub fn next(&mut self) {
        if !self.is_at_end() && self.is_at_viewport_end() {
            self.start_position += 1;
        } else if !self.is_at_viewport_end() {
            self.selected_position += 1;
        }
    }

    pub fn prev(&mut self) {
        if !self.is_at_beginning() && self.is_at_viewport_start() {
            self.start_position -= 1;
        } else if !self.is_at_viewport_start() {
            self.selected_position -= 1;
        }
    }

    fn is_at_viewport_end(&self) -> bool {
        self.viewport_length == 0 || self.selected_position == self.viewport_length - 1
    }

    fn is_at_viewport_start(&self) -> bool {
        self.selected_position == 0
    }

    fn is_at_end(&self) -> bool {
        self.content_length == 0
            || (self.start_position == self.content_length - self.viewport_length
                && self.selected_position == self.viewport_length - 1)
    }

    fn is_at_beginning(&self) -> bool {
        self.start_position == 0 && self.selected_position == 0
    }
}

#[cfg(test)]
mod test {
    use ratatui::{
        buffer::Buffer,
        layout::Rect,
        style::{Style, Stylize},
        widgets::StatefulWidget,
    };

    use super::{TimeBar, TimeBarState, TimeUnit};

    const X0: u16 = 10;
    const Y0: u16 = 10;

    #[test]
    pub fn test_render() {
        let (time_bar, mut state, mut buf, area) = setup(50);

        time_bar.render(buf.area, &mut buf, &mut state);

        let mut expected = Buffer::with_lines(vec![
            "╻0 ns     ╻10 ns    ╻20 ns    ╻30 ns    ╻40 ns    ",
            "┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷",
        ]);
        expected.area = area;
        expected.set_style(Rect::new(X0, Y0, 1, 2), Style::default().on_red());

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
            "╻0 ns     ╻10 ns    ╻20 ns    ╻30 ns    ╻40 ns    ",
            "┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷",
        ]);
        expected.area = area;
        expected.set_style(Rect::new(X0 + 10, Y0, 1, 2), Style::default().on_red());

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
            "    ╻10 ns    ╻20 ns    ╻30 ns    ╻40 ns    ╻50 ns",
            "┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻",
        ]);
        expected.area = area;
        expected.set_style(Rect::new(X0 + 49, Y0, 1, 2), Style::default().on_red());

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
            "    ╻10 ns    ╻20 ns    ╻30 ns    ╻40 ns    ╻50 ns",
            "┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻",
        ]);
        expected.area = area;
        expected.set_style(Rect::new(X0 + 44, Y0, 1, 2), Style::default().on_red());

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
            "     ╻10 ns    ╻20 ns    ╻30 ns    ╻40 ns    ╻50 n",
            "┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷",
        ]);
        expected.area = area;
        expected.set_style(Rect::new(X0, Y0, 1, 2), Style::default().on_red());

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
            "   ╻10 ns    ╻20 ns    ╻30 ns    ╻40 ns    ╻50 ns",
            "┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻",
        ]);
        expected.area = area;
        expected.set_style(Rect::new(X0 + 48, Y0, 1, 2), Style::default().on_red());

        assert_eq!(buf, expected);
    }

    fn setup(viewport_length: usize) -> (TimeBar, TimeBarState, Buffer, Rect) {
        let time_bar = TimeBar::default()
            .time_unit(TimeUnit::Nanoseconds)
            .tick_period(10.0)
            .tick_count(10);
        let mut state = TimeBarState::new(100);

        let area = Rect::new(X0, Y0, viewport_length as u16, 2);
        let buf = Buffer::empty(area);
        state.set_viewport_length(viewport_length);

        (time_bar, state, buf, area)
    }
}
