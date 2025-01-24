use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{palette::tailwind::SLATE, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, ScrollbarState, StatefulWidget},
};

use crate::widgets::{TimeBar, TimeBarState, Waveform, WaveformScrollState};

use super::models::{SimulationSpec, WaveSpec};

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const NUMBER_OF_CELLS_PER_UNIT_TIME: usize = 3;

#[derive(Default)]
pub struct WaveViewer {
    simulation: SimulationSpec,
    list_state: ListState,
    selected_idx: Option<usize>,
    waveform_scroll_state: WaveformScrollState,
    timebar_state: TimeBarState,
    horizontal_scrollbar_state: ScrollbarState,
    horizontal_content_length: usize,
    horizontal_position: usize,
}

impl WaveViewer {
    pub fn simulation(mut self, simulation: SimulationSpec) -> Self {
        self.simulation = simulation;
        self
    }

    pub fn set_simulation(&mut self, simulation: SimulationSpec) {
        self.simulation = simulation;
        if !self.simulation.wave_specs.is_empty() {
            self.list_state.select_first();
            self.selected_idx = Some(0);
            self.horizontal_content_length = NUMBER_OF_CELLS_PER_UNIT_TIME
                * 2usize.pow(self.simulation.zoom as u32)
                * self.simulation.total_time;
            self.horizontal_scrollbar_state =
                ScrollbarState::new(self.horizontal_content_length).position(0);
            self.waveform_scroll_state
                .set_content_length(self.horizontal_content_length);
            self.timebar_state
                .set_content_length(self.horizontal_content_length);
        }
    }

    pub fn scroll_right(&mut self) {
        self.horizontal_scrollbar_state.next();
        self.waveform_scroll_state.next();
        self.timebar_state.next();
        if self.horizontal_position + 1 < self.horizontal_content_length {
            self.horizontal_position += 1;
        }
    }

    pub fn scroll_left(&mut self) {
        self.horizontal_scrollbar_state.prev();
        self.waveform_scroll_state.prev();
        self.timebar_state.prev();
        if self.horizontal_position != 0 {
            self.horizontal_position -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        if let Some(idx) = self.selected_idx {
            self.list_state.select_next();
            let new_idx = usize::saturating_add(idx, 1);
            self.selected_idx = Some(usize::min(self.simulation.wave_specs.len() - 1, new_idx));
        }
    }

    pub fn scroll_up(&mut self) {
        if let Some(idx) = self.selected_idx {
            self.list_state.select_previous();
            self.selected_idx = Some(usize::saturating_sub(idx, 1));
        }
    }

    pub fn get_highlighted_unit_time(&self) -> usize {
        self.horizontal_position
            / (NUMBER_OF_CELLS_PER_UNIT_TIME * 2usize.pow(self.simulation.zoom as u32))
    }

    pub fn render_mut(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        let mut waveform_scroll_state = self.waveform_scroll_state.clone();
        let items = self.new_list_items(rect.width, &mut waveform_scroll_state);
        self.waveform_scroll_state = waveform_scroll_state;
        let list = List::new(items);

        let (tick_count, tick_period) = self.calculate_preferred_tick();
        let time_bar = TimeBar::default()
            .tick_count(tick_count)
            .tick_period(tick_period);

        let chunks = Layout::vertical(vec![Constraint::Min(0), Constraint::Length(2)]).split(rect);
        f.render_stateful_widget(list, chunks[0], &mut self.list_state);
        f.render_stateful_widget(time_bar, chunks[1], &mut self.timebar_state);
    }

    fn calculate_preferred_tick(&self) -> (usize, f64) {
        let multiplier = Self::nearest_power_of_2_multiplier(NUMBER_OF_CELLS_PER_UNIT_TIME, 10);
        let tick_count = NUMBER_OF_CELLS_PER_UNIT_TIME * multiplier;
        let tick_period = multiplier as f64 / 2usize.pow(self.simulation.zoom as u32) as f64;
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
        waveform_scroll_state: &mut WaveformScrollState,
    ) -> Vec<ListItem<'a>> {
        self.simulation
            .wave_specs
            .iter()
            .enumerate()
            .map(|(i, ws)| {
                self.new_list_item(
                    ws,
                    waveform_scroll_state,
                    Some(i) == self.selected_idx,
                    render_area_width,
                )
            })
            .collect()
    }

    fn new_list_item<'a>(
        &self,
        wave_spec: &WaveSpec,
        waveform_scroll_state: &mut WaveformScrollState,
        is_selected: bool,
        render_area_width: u16,
    ) -> ListItem<'a> {
        let waveform = Waveform::new(wave_spec)
            .zoom(self.simulation.zoom)
            .block(Block::new().borders(Borders::BOTTOM))
            .selected_style(SELECTED_STYLE)
            .selected(is_selected);
        let list_item_height = wave_spec.height * 2 + 2;
        let mut draw_buffer = Buffer::empty(Rect::new(0, 0, render_area_width, list_item_height));
        waveform.render(draw_buffer.area, &mut draw_buffer, waveform_scroll_state);
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
}
