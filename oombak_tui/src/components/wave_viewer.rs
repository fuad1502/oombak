use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget},
};

use crate::{
    styles::wave_viewer::SELECTED_WAVEFORM_STYLE,
    widgets::{ScrollState, TimeBar, Waveform},
};

use super::models::{SimulationSpec, WaveSpec};

const NUMBER_OF_CELLS_PER_UNIT_TIME: usize = 3;

#[derive(Default)]
pub struct WaveViewer {
    simulation: Arc<RwLock<SimulationSpec>>,
    list_state: ListState,
    selected_idx: Option<usize>,
    scroll_state: ScrollState,
    horizontal_content_length: usize,
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
            self.update_content_length();
        } else {
            self.selected_idx = None;
        }
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
        self.update_content_length();
    }

    pub fn zoom_out(&mut self) {
        {
            let mut simulation = self.get_simulation_mut();
            simulation.zoom = simulation.zoom.saturating_sub(1);
        }
        self.update_content_length();
    }

    pub fn get_highlighted_unit_time(&self) -> usize {
        let absolute_highlight_position =
            self.scroll_state.start_position() + self.scroll_state.selected_position();
        let zoom = self.get_simulation().zoom;
        absolute_highlight_position / (NUMBER_OF_CELLS_PER_UNIT_TIME * 2usize.pow(zoom as u32))
    }

    pub fn render_mut(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        let mut scroll_state = self.scroll_state;
        let items = self.new_list_items(rect.width, &mut scroll_state);
        self.scroll_state = scroll_state;
        let list = List::new(items);

        let (tick_count, tick_period) = self.calculate_preferred_tick();
        let time_bar = TimeBar::default()
            .tick_count(tick_count)
            .tick_period(tick_period);

        let chunks = Layout::vertical(vec![Constraint::Min(0), Constraint::Length(3)]).split(rect);
        f.render_stateful_widget(list, chunks[0], &mut self.list_state);
        f.render_stateful_widget(time_bar, chunks[1], &mut self.scroll_state);
    }

    fn update_content_length(&mut self) {
        let zoom = self.get_simulation().zoom;
        let total_time = self.get_simulation().total_time;
        self.horizontal_content_length =
            NUMBER_OF_CELLS_PER_UNIT_TIME * 2usize.pow(zoom as u32) * total_time;
        self.scroll_state
            .set_content_length(self.horizontal_content_length);
    }

    fn calculate_preferred_tick(&self) -> (usize, f64) {
        let multiplier = Self::nearest_power_of_2_multiplier(NUMBER_OF_CELLS_PER_UNIT_TIME, 10);
        let tick_count = NUMBER_OF_CELLS_PER_UNIT_TIME * multiplier;
        let zoom = self.get_simulation().zoom;
        let tick_period = multiplier as f64 / 2usize.pow(zoom as u32) as f64;
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
        self.get_simulation()
            .wave_specs
            .iter()
            .enumerate()
            .map(|(i, ws)| {
                self.new_list_item(
                    ws,
                    scroll_state,
                    Some(i) == self.selected_idx,
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
        render_area_width: u16,
    ) -> ListItem<'a> {
        let waveform = Waveform::new(wave_spec)
            .zoom(self.get_simulation().zoom)
            .block(Block::new().borders(Borders::BOTTOM))
            .selected_style(SELECTED_WAVEFORM_STYLE)
            .selected(is_selected);
        let list_item_height = wave_spec.height * 2 + 2;
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
}
