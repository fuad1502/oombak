use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::StatefulWidget};

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

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {}
}

#[derive(Default)]
pub struct TimeBarState {
    total_time: usize,
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
