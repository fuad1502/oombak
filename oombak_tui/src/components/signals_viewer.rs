use std::sync::{Arc, RwLock, RwLockReadGuard};

use ratatui::{
    style::Style,
    symbols,
    text::{Line, Span},
    widgets::{List, ListItem, ListState},
};

use crate::{
    styles::signals_viewer::{
        SELECTED_SIGNAL_STYLE, SIGNAL_NAME_STYLE, SIGNAL_VALUE_STYLE, SIGNAL_WIDTH_STYLE,
    },
    utils::{self, bitvec_str},
};

use super::models::{SimulationSpec, WaveSpec};

#[derive(Default)]
pub struct SignalsViewer {
    simulation: Arc<RwLock<SimulationSpec>>,
    list_state: ListState,
    selected_idx: Option<usize>,
    highlight_idx: usize,
}

impl SignalsViewer {
    pub fn simulation(mut self, simulation: Arc<RwLock<SimulationSpec>>) -> Self {
        self.simulation = simulation;
        self
    }

    pub fn set_highlight(&mut self, idx: usize) {
        self.highlight_idx = idx;
    }

    pub fn reload(&mut self) {
        if !self.get_simulation().wave_specs.is_empty() {
            self.selected_idx = Some(0);
            self.list_state.select_first();
        } else {
            self.selected_idx = None;
        }
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

    pub fn selected_signal_name(&self) -> Option<String> {
        self.selected_idx.map(|i| {
            self.get_simulation().wave_specs[i]
                .wave
                .signal_name
                .to_string()
        })
    }

    pub fn render_mut(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        let items = self.create_list_items(rect.width);
        let list = List::new(items);
        f.render_stateful_widget(list, rect, &mut self.list_state);
    }

    fn create_list_items<'a>(&self, width: u16) -> Vec<ListItem<'a>> {
        self.get_simulation()
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
        let horizontal_line = Self::create_horizontal_line(width);
        lines[list_item_height / 2] = self.new_signal_description(wave_spec, style);
        lines.push(horizontal_line);
        lines.into()
    }

    fn new_signal_description<'a>(&self, wave_spec: &WaveSpec, style: Style) -> Line<'a> {
        let signal_name = Span::from(wave_spec.wave.signal_name.clone()).style(SIGNAL_NAME_STYLE);
        let signal_width =
            Span::from(format!("[{}:0]", wave_spec.wave.width - 1)).style(SIGNAL_WIDTH_STYLE);
        let signal_value = Span::from(self.get_highlighted_value_of(wave_spec).to_string())
            .style(SIGNAL_VALUE_STYLE);
        Line::from(vec![
            Span::from(" "),
            signal_name,
            Span::from(" "),
            signal_width,
            Span::from(" ("),
            signal_value,
            Span::from(")"),
        ])
        .style(style)
    }

    fn get_highlighted_value_of(&self, wave_spec: &WaveSpec) -> String {
        if let Some(value) = wave_spec.wave.at(self.highlight_idx) {
            let option = bitvec_str::Option::from(wave_spec);
            let prefix = match option.radix {
                bitvec_str::Radix::Binary => "0b",
                bitvec_str::Radix::Hexadecimal => "0x",
                bitvec_str::Radix::Octal => "0o",
                bitvec_str::Radix::Decimal => "",
            };
            format!("{prefix}{}", utils::bitvec_str::from(value, &option))
        } else {
            "x".to_string()
        }
    }

    fn create_horizontal_line<'a>(width: u16) -> Line<'a> {
        symbols::line::HORIZONTAL.repeat(width as usize).into()
    }

    fn get_simulation(&self) -> RwLockReadGuard<'_, SimulationSpec> {
        self.simulation.read().unwrap()
    }
}
