use std::sync::{mpsc::Sender, Arc, RwLock};

use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

use crate::{
    component::{Component, HandleResult},
    threads::RendererMessage,
    widgets::KeyMaps,
};

use super::{selector::Selector, signal_value_setter::SignalValueSetter};

pub struct SignalPropertiesEditor {
    selector: Selector,
}

impl SignalPropertiesEditor {
    pub fn new(renderer_channel: Sender<RendererMessage>) -> Self {
        let selection: Vec<(String, Arc<RwLock<dyn Component>>)> = vec![
            (
                "Set signal value".to_string(),
                Arc::new(RwLock::new(SignalValueSetter::new(
                    renderer_channel.clone(),
                ))),
            ),
            (
                "Set periodic signal value".to_string(),
                Arc::new(RwLock::new(SignalValueSetter::new(
                    renderer_channel.clone(),
                ))),
            ),
        ];
        Self {
            selector: Selector::new(selection, renderer_channel),
        }
    }

    pub fn set_signal_name(&mut self, signal_name: &str) {
        self.selector.set_title(format!("[{signal_name}]"));
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
