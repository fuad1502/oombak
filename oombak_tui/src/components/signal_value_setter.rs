use std::{
    collections::HashMap,
    sync::{mpsc::Sender, Arc, RwLock},
};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, widgets::Paragraph, Frame};

use crate::{
    component::{Component, HandleResult},
    threads::RendererMessage,
    widgets::KeyMaps,
};

pub struct SignalValueSetter {
    renderer_channel: Sender<RendererMessage>,
}

impl SignalValueSetter {
    pub fn new(renderer_channel: Sender<RendererMessage>) -> Self {
        Self { renderer_channel }
    }
}

impl Component for SignalValueSetter {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let paragraph = Paragraph::new("Halo!");
        f.render_widget(paragraph, rect);
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> HandleResult {
        match key_event.code {
            KeyCode::Char('q') => return HandleResult::ReleaseFocus,
            _ => (),
        }
        self.renderer_channel.send(RendererMessage::Render).unwrap();
        HandleResult::Handled
    }

    fn handle_resize_event(&mut self, _: u16, _: u16) -> HandleResult {
        HandleResult::NotHandled
    }

    fn handle_focus_gained(&mut self) -> HandleResult {
        unimplemented!()
    }

    fn get_focused_child(&self) -> Option<Arc<RwLock<dyn Component>>> {
        None
    }

    fn get_key_mappings(&self) -> KeyMaps {
        KeyMaps::from(HashMap::new())
    }
}
