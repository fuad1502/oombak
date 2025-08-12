use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget},
    Frame,
};

use crate::{
    styles::wave_viewer::SELECTED_WAVEFORM_STYLE,
    widgets::{ScrollState, TimeBar, Waveform},
};

use super::models::{SimulationSpec, WaveSpec};

const NUMBER_OF_CELLS_PER_UNIT_TIME: usize = 1;

#[derive(Default)]
pub struct WaveViewer {
    simulation: Arc<RwLock<SimulationSpec>>,
    list_state: ListState,
    selected_idx: Option<usize>,
    scroll_state: ScrollState,
}

impl WaveViewer {
    pub fn simulation(mut self, simulation: Arc<RwLock<SimulationSpec>>) -> Self {
        self.simulation = simulation;
        self
    }

    pub fn reload(&mut self) {
        if !self.get_simulation().wave_specs.is_empty() {
            self.list_state.select_first();
            self.selected_idx = Some(0);
        } else {
            self.selected_idx = None;
        }
        self.update_scroll_state_content_length();
    }

    pub fn scroll_right(&mut self) {
        self.scroll_state.next();
    }

    pub fn scroll_left(&mut self) {
        self.scroll_state.prev();
    }

    pub fn scroll_down(&mut self) {
        if let Some(idx) = self.selected_idx {
            self.list_state.select_next();
            let number_of_waves = self.get_simulation().wave_specs.len();
            self.selected_idx = Some(usize::min(number_of_waves - 1, idx + 1));
        }
    }

    pub fn scroll_up(&mut self) {
        if let Some(idx) = self.selected_idx {
            self.list_state.select_previous();
            self.selected_idx = Some(usize::saturating_sub(idx, 1));
        }
    }

    pub fn zoom_in(&mut self) {
        {
            let mut simulation = self.get_simulation_mut();
            simulation.zoom = simulation.zoom.saturating_add(1);
        }
        self.update_scroll_state_content_length();
    }

    pub fn zoom_out(&mut self) {
        {
            let mut simulation = self.get_simulation_mut();
            simulation.zoom = simulation.zoom.saturating_sub(1);
        }
        self.update_scroll_state_content_length();
    }

    pub fn get_highlighted_unit_time(&self) -> usize {
        let absolute_highlight_position =
            self.scroll_state.start_position() + self.scroll_state.selected_position();
        absolute_highlight_position / self.unit_width()
    }

    pub fn render_mut(&mut self, f: &mut Frame, rect: Rect) {
        let mut scroll_state = self.scroll_state;
        let items = self.new_list_items(rect.width, &mut scroll_state);
        self.scroll_state = scroll_state;
        let list = List::new(items);

        let areas = Layout::vertical(vec![Constraint::Min(0), Constraint::Length(3)]).split(rect);
        f.render_stateful_widget(list, areas[0], &mut self.list_state);
        self.render_time_bar(f, areas[1]);
    }

    fn render_time_bar(&mut self, f: &mut Frame, area: Rect) {
        let (tick_count, tick_period) = self.calculate_preferred_tick();
        let time_bar = TimeBar::default()
            .tick_count(tick_count)
            .tick_period(tick_period);

        let block = Block::new().borders(Borders::LEFT);
        f.render_stateful_widget(time_bar, block.inner(area), &mut self.scroll_state);
        f.render_widget(block, area);
    }

    fn update_scroll_state_content_length(&mut self) {
        let total_time = self.get_simulation().total_time;
        let content_length = self.unit_width() * total_time;
        self.scroll_state.set_content_length(content_length);
    }

    fn calculate_preferred_tick(&self) -> (usize, f64) {
        let multiplier = Self::nearest_power_of_2_multiplier(NUMBER_OF_CELLS_PER_UNIT_TIME, 16);
        let tick_count = NUMBER_OF_CELLS_PER_UNIT_TIME * multiplier;
        let tick_period =
            multiplier as f64 / (self.unit_width() / NUMBER_OF_CELLS_PER_UNIT_TIME) as f64;
        (tick_count, tick_period)
    }

    fn nearest_power_of_2_multiplier(x: usize, target: usize) -> usize {
        let first = usize::next_power_of_two(target / x);
        let second = first / 2;
        if (first * x).abs_diff(target) < (second * x).abs_diff(target) {
            first
        } else {
            second
        }
    }

    fn new_list_items<'a>(
        &self,
        render_area_width: u16,
        scroll_state: &mut ScrollState,
    ) -> Vec<ListItem<'a>> {
        let number_of_items = self.get_simulation().wave_specs.len();
        self.get_simulation()
            .wave_specs
            .iter()
            .enumerate()
            .map(|(i, ws)| {
                self.new_list_item(
                    ws,
                    scroll_state,
                    Some(i) == self.selected_idx,
                    i == number_of_items - 1,
                    render_area_width,
                )
            })
            .collect()
    }

    fn new_list_item<'a>(
        &self,
        wave_spec: &WaveSpec,
        scroll_state: &mut ScrollState,
        is_selected: bool,
        is_last: bool,
        render_area_width: u16,
    ) -> ListItem<'a> {
        let border_set = symbols::border::Set {
            bottom_left: symbols::line::NORMAL.cross,
            ..symbols::border::PLAIN
        };
        let block = Block::new()
            .borders(Borders::BOTTOM | Borders::LEFT)
            .border_set(border_set);
        let waveform = Waveform::new(wave_spec)
            .zoom(self.get_simulation().zoom)
            .block(block)
            .selected_style(SELECTED_WAVEFORM_STYLE)
            .selected(is_selected);
        let waveform = if is_last {
            waveform
        } else {
            waveform.extend_cursor_highlight()
        };
        let list_item_height = (wave_spec.height * 2 + 1) + 1;
        let mut draw_buffer = Buffer::empty(Rect::new(0, 0, render_area_width, list_item_height));
        waveform.render(draw_buffer.area, &mut draw_buffer, scroll_state);
        ListItem::from(Self::buffer_to_lines(&draw_buffer))
    }

    fn buffer_to_lines<'a>(buffer: &Buffer) -> Vec<Line<'a>> {
        let mut lines = vec![];
        for i in 0..buffer.area.height {
            lines.push(Self::get_buffer_line(buffer, i));
        }
        lines
    }

    fn get_buffer_line<'a>(buffer: &Buffer, y: u16) -> Line<'a> {
        let mut line = Line::default();
        for i in 0..buffer.area.width {
            let cell = buffer.cell((i, y)).unwrap();
            let symbol = cell.symbol().to_string();
            let style = cell.style();
            line += Span::from(symbol).style(style);
        }
        line
    }

    fn get_simulation(&self) -> RwLockReadGuard<'_, SimulationSpec> {
        self.simulation.read().unwrap()
    }

    fn get_simulation_mut(&mut self) -> RwLockWriteGuard<'_, SimulationSpec> {
        self.simulation.write().unwrap()
    }

    fn unit_width(&self) -> usize {
        let zoom = self.get_simulation().zoom;
        NUMBER_OF_CELLS_PER_UNIT_TIME * 2usize.pow(zoom as u32)
    }
}
