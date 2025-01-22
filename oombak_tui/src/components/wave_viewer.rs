use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{palette::tailwind::SLATE, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Widget},
};

use crate::{
    component::{Component, HandleResult},
    widgets::Waveform,
};

use super::models::{SimulationSpec, WaveSpec};

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

#[derive(Default)]
pub struct WaveViewer {
    simulation: SimulationSpec,
    list_state: ListState,
    selected_idx: Option<usize>,
    highlight_idx: u16,
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
        }
    }

    pub fn set_highlight(&mut self, idx: u16) {
        self.highlight_idx = idx;
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
}

impl Component for WaveViewer {
    fn render_mut(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        let items = self.new_list_items(rect.width);
        let list = List::new(items);
        f.render_stateful_widget(list, rect, &mut self.list_state);
    }

    fn handle_key_event(&mut self, _key_event: &crossterm::event::KeyEvent) -> HandleResult {
        HandleResult::NotHandled
    }

    fn try_propagate_event(&mut self, _event: &crossterm::event::Event) -> HandleResult {
        HandleResult::NotHandled
    }

    fn set_focus_to_self(&mut self) {}

    fn render(&self, _f: &mut ratatui::Frame, _rect: Rect) {}
}

impl WaveViewer {
    fn new_list_items<'a>(&self, render_area_width: u16) -> Vec<ListItem<'a>> {
        self.simulation
            .wave_specs
            .iter()
            .enumerate()
            .map(|(i, ws)| self.new_list_item(ws, Some(i) == self.selected_idx, render_area_width))
            .collect()
    }

    fn new_list_item<'a>(
        &self,
        wave_spec: &WaveSpec,
        is_selected: bool,
        render_area_width: u16,
    ) -> ListItem<'a> {
        let waveform = Waveform::from(wave_spec)
            .width(self.simulation.zoom)
            .highlight(self.highlight_idx)
            .block(Block::new().borders(Borders::BOTTOM))
            .selected_style(SELECTED_STYLE)
            .selected(is_selected);
        let list_item_height = wave_spec.height * 2 + 2;
        let mut draw_buffer = Buffer::empty(Rect::new(0, 0, render_area_width, list_item_height));
        waveform.render(draw_buffer.area, &mut draw_buffer);
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
