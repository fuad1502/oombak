use ratatui::{
    style::Style,
    symbols,
    text::Line,
    widgets::{List, ListItem, ListState},
};

use crate::{
    styles::signals_viewer::SELECTED_SIGNAL_STYLE,
    utils::{self, bitvec_str},
};

use super::models::{SimulationSpec, WaveSpec};

#[derive(Default)]
pub struct SignalsViewer {
    simulation: SimulationSpec,
    list_state: ListState,
    selected_idx: Option<usize>,
    highlight_idx: usize,
}

impl SignalsViewer {
    pub fn simulation(mut self, simulation: SimulationSpec) -> Self {
        self.simulation = simulation;
        self
    }

    pub fn set_highlight(&mut self, idx: usize) {
        self.highlight_idx = idx;
    }

    pub fn set_simulation(&mut self, simulation: SimulationSpec) {
        self.simulation = simulation;
        if !self.simulation.wave_specs.is_empty() {
            self.selected_idx = Some(0);
            self.list_state.select_first();
        } else {
            self.selected_idx = None;
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

    pub fn render_mut(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        let items = self.create_list_items(rect.width);
        let list = List::new(items);
        f.render_stateful_widget(list, rect, &mut self.list_state);
    }

    fn create_list_items<'a>(&self, width: u16) -> Vec<ListItem<'a>> {
        self.simulation
            .wave_specs
            .iter()
            .enumerate()
            .map(|(i, spec)| {
                if Some(i) == self.selected_idx {
                    self.new_list_item(spec, width, SELECTED_SIGNAL_STYLE)
                } else {
                    self.new_list_item(spec, width, Style::default())
                }
            })
            .collect()
    }

    fn new_list_item<'a>(&self, wave_spec: &WaveSpec, width: u16, style: Style) -> ListItem<'a> {
        let list_item_height = (wave_spec.height * 2 + 1) as usize;
        let mut lines = vec![Line::from(" ").style(style); list_item_height];
        let signal_description_line =
            Line::from(self.new_signal_description(wave_spec)).style(style);
        let horizontal_line = Self::create_horizontal_line(width);
        lines[list_item_height / 2] = signal_description_line;
        lines.push(horizontal_line);
        lines.into()
    }

    fn new_signal_description(&self, wave_spec: &WaveSpec) -> String {
        format!(
            "{} [{}:0] ({})",
            wave_spec.wave.signal_name,
            wave_spec.wave.width,
            self.get_highlighted_value_of(wave_spec)
        )
    }

    fn get_highlighted_value_of(&self, wave_spec: &WaveSpec) -> String {
        if let Some((idx, _)) = wave_spec.wave.value_idx_at(self.highlight_idx) {
            let value = &wave_spec.wave.values[idx].0;
            let option = bitvec_str::Option::from(wave_spec);
            utils::bitvec_str::from(value, &option)
        } else {
            "x".to_string()
        }
    }

    fn create_horizontal_line<'a>(width: u16) -> Line<'a> {
        symbols::line::HORIZONTAL.repeat(width as usize).into()
    }
}
