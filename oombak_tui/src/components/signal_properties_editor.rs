use std::sync::{mpsc::Sender, Arc, RwLock};

use crossterm::event::KeyEvent;
use oombak_sim::response::LoadedDut;
use ratatui::{layout::Rect, Frame};

use crate::{
    component::{Component, HandleResult},
    threads::RendererMessage,
    widgets::KeyMaps,
};

use super::{
    periodic_signal_setter::PeriodicSignalSetter,
    selector::{Selection, Selector},
    signal_value_setter::SignalValueSetter,
};

pub struct SignalPropertiesEditor {
    selector: Selector,
    signal_name: Option<String>,
    input_ports: Vec<String>,
}

impl SignalPropertiesEditor {
    pub fn new(renderer_channel: Sender<RendererMessage>) -> Self {
        let selection = vec![
            Selection::new(
                "Set signal value",
                Arc::new(RwLock::new(SignalValueSetter::new(
                    renderer_channel.clone(),
                ))),
            ),
            Selection::new(
                "Set periodic signal value",
                Arc::new(RwLock::new(PeriodicSignalSetter::new(
                    renderer_channel.clone(),
                ))),
            ),
        ];
        Self {
            selector: Selector::new(selection, renderer_channel),
            signal_name: None,
            input_ports: vec![],
        }
    }

    pub fn set_signal_name(&mut self, signal_name: &str) {
        self.signal_name = Some(signal_name.to_string());
        self.selector.set_title(format!("[{signal_name}]"));
        self.enable_or_disable_selections();
    }

    pub fn set_loaded_dut(&mut self, loaded_dut: &LoadedDut) {
        self.set_input_ports(loaded_dut);
        self.enable_or_disable_selections();
    }

    fn set_input_ports(&mut self, loaded_dut: &LoadedDut) {
        self.input_ports.clear();
        for port in loaded_dut.root_node.get_ports() {
            if port.is_input_port() {
                self.input_ports.push(port.name.clone());
            }
        }
    }

    fn enable_or_disable_selections(&mut self) {
        if self.is_signal_settable() {
            self.selector.enable_selection(0);
            self.selector.enable_selection(1);
        } else {
            self.selector.disable_selection(0);
            self.selector.disable_selection(1);
        }
    }

    fn is_signal_settable(&self) -> bool {
        if let Some(name) = &self.signal_name {
            self.input_ports.contains(name)
        } else {
            false
        }
    }
}

impl Component for SignalPropertiesEditor {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        self.selector.render(f, rect);
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> HandleResult {
        self.selector.handle_key_event(key_event)
    }

    fn handle_resize_event(&mut self, columns: u16, rows: u16) -> HandleResult {
        self.selector.handle_resize_event(columns, rows)
    }

    fn handle_focus_gained(&mut self) -> HandleResult {
        self.selector.handle_focus_gained()
    }

    fn get_focused_child(&self) -> Option<Arc<RwLock<dyn Component>>> {
        self.selector.get_focused_child()
    }

    fn get_key_mappings(&self) -> KeyMaps {
        self.selector.get_key_mappings()
    }
}
