use ratatui::{
    style::{palette::tailwind::SLATE, Modifier, Style},
    symbols,
    text::Line,
    widgets::{List, ListItem, ListState},
};

use crate::{
    component::{Component, HandleResult},
    utils::{self, bitvec_str},
};

use super::models::{SimulationSpec, WaveSpec};

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

#[derive(Default)]
pub struct SignalsViewer {
    simulation: SimulationSpec,
    list_state: ListState,
    selected_idx: Option<usize>,
    highlight_idx: u16,
}

impl SignalsViewer {
    pub fn simulation(mut self, simulation: SimulationSpec) -> Self {
        self.simulation = simulation;
        self
    }

    pub fn set_highlight(&mut self, idx: u16) {
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
}

impl Component for SignalsViewer {
    fn render_mut(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        let items = self.create_list_items(rect.width);
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

    fn render(&self, _f: &mut ratatui::Frame, _rect: ratatui::prelude::Rect) {}
}

impl SignalsViewer {
    fn create_list_items<'a>(&self, width: u16) -> Vec<ListItem<'a>> {
        self.simulation
            .wave_specs
            .iter()
            .enumerate()
            .map(|(i, spec)| {
                if Some(i) == self.selected_idx {
                    self.new_list_item(spec, width, SELECTED_STYLE)
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
        let waveform_length = wave_spec.height * 2 + 1 + self.simulation.zoom as u16;
        if let Some(value) = wave_spec
            .wave
            .values
            .get(self.highlight_idx as usize / waveform_length as usize)
        {
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
