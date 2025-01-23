use ratatui::{
    buffer::Buffer, layout::Rect, style::Style, style::Stylize, text::Line, widgets::StatefulWidget,
};

#[derive(Default)]
pub struct TimeBar {
    tick_period: usize,
    time_unit: TimeUnit,
    highlight_style: Style,
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

    pub fn tick_period(mut self, tick_period: usize) -> Self {
        self.tick_period = tick_period;
        self
    }
}

impl StatefulWidget for TimeBar {
    type State = TimeBarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.set_viewport_length(area.width as usize);
        let number_of_ticks = state.viewport_length / self.tick_period;
        let floored_start_position = state.start_position / self.tick_period * self.tick_period;

        let mut line_1 = String::new();
        let mut line_2 = String::new();
        for i in 0..number_of_ticks + 1 {
            let time = floored_start_position + i * self.tick_period;
            let segment = format!("╻{0:1$}", self.format(time), self.tick_period - 1);
            line_1 += &segment;
            let sub_ticks_left = usize::saturating_sub(self.tick_period, 2) / 2;
            let sub_ticks_right = sub_ticks_left;
            let middle_ticks = self.tick_period - sub_ticks_left - sub_ticks_right - 1;
            let segment = format!(
                "┻{0:┷<1$}{0:┻<2$}{0:┷<3$}",
                "", sub_ticks_left, middle_ticks, sub_ticks_right
            );
            line_2 += &segment;
        }

        let skip_start = state.start_position - floored_start_position;
        let line_1: String = line_1
            .chars()
            .skip(skip_start)
            .take(state.viewport_length)
            .collect();
        let line_1 = Line::from(line_1);
        let line_2: String = line_2
            .chars()
            .skip(skip_start)
            .take(state.viewport_length)
            .collect();
        let line_2 = Line::from(line_2);

        buf.set_line(0, 0, &line_1, area.width);
        buf.set_line(0, 1, &line_2, area.width);

        buf.set_style(
            Rect::new(state.selected_position as u16, 0, 1, 2),
            Style::default().on_red(),
        );
    }
}

impl TimeBar {
    fn format(&self, time: usize) -> String {
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
    total_time: usize,
    start_position: usize,
    selected_position: usize,
    viewport_length: usize,
}

impl TimeBarState {
    pub fn new(total_time: usize) -> Self {
        Self {
            total_time,
            ..Default::default()
        }
    }

    pub fn set_viewport_length(&mut self, viewport_length: usize) {
        self.viewport_length = viewport_length;
    }

    pub fn next(&mut self) {}

    pub fn prev(&mut self) {}
}

mod test {
    use ratatui::{
        buffer::Buffer,
        layout::Rect,
        style::{Style, Stylize},
        widgets::StatefulWidget,
    };

    use super::{TimeBar, TimeBarState, TimeUnit};

    #[test]
    pub fn test_render() {
        let (time_bar, mut state, mut buf) = setup();

        time_bar.render(buf.area, &mut buf, &mut state);

        let mut expected = Buffer::with_lines(vec![
            "╻0 ns     ╻10 ns    ╻20 ns    ╻30 ns    ╻40 ns    ",
            "┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷",
        ]);
        expected.set_style(Rect::new(0, 0, 1, 2), Style::default().on_red());

        assert_eq!(buf, expected);
    }

    #[test]
    pub fn test_scroll_little() {
        let (time_bar, mut state, mut buf) = setup();

        for _ in 0..10 {
            state.next();
        }

        time_bar.render(buf.area, &mut buf, &mut state);

        let mut expected = Buffer::with_lines(vec![
            "╻0 ns     ╻10 ns    ╻20 ns    ╻30 ns    ╻40 ns    ",
            "┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷",
        ]);
        expected.set_style(Rect::new(10, 0, 1, 2), Style::default().on_red());

        assert_eq!(buf, expected);
    }

    #[test]
    pub fn test_scroll_pass_end() {
        let (time_bar, mut state, mut buf) = setup();

        for _ in 0..55 {
            state.next();
        }

        time_bar.render(buf.area, &mut buf, &mut state);

        let mut expected = Buffer::with_lines(vec![
            "    ╻10 ns    ╻20 ns    ╻30 ns    ╻40 ns    ╻50 ns",
            "┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻┷┷┷┷┻",
        ]);
        expected.set_style(Rect::new(50, 0, 1, 2), Style::default().on_red());

        assert_eq!(buf, expected);
    }

    #[test]
    pub fn test_scroll_pass_end_and_back() {
        let (time_bar, mut state, mut buf) = setup();

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
        expected.set_style(Rect::new(45, 0, 1, 2), Style::default().on_red());

        assert_eq!(buf, expected);
    }

    #[test]
    pub fn test_scroll_pass_end_and_pass_back() {
        let (time_bar, mut state, mut buf) = setup();

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
        expected.set_style(Rect::new(0, 0, 1, 2), Style::default().on_red());

        assert_eq!(buf, expected);
    }

    fn setup() -> (TimeBar, TimeBarState, Buffer) {
        let time_bar = TimeBar::default()
            .time_unit(TimeUnit::Nanoseconds)
            .tick_period(10);
        let state = TimeBarState::new(100);

        let buf = Buffer::empty(Rect::new(0, 0, 50, 2));

        (time_bar, state, buf)
    }
}
