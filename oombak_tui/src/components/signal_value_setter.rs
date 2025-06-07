use std::{
    collections::HashMap,
    sync::{mpsc::Sender, Arc, RwLock},
};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, Frame};

use crate::{
    component::{Component, HandleResult},
    threads::RendererMessage,
    widgets::{Form, FormState, InputField, KeyMaps},
};

pub struct SignalValueSetter {
    renderer_channel: Sender<RendererMessage>,
    form_state: FormState,
}

impl SignalValueSetter {
    pub fn new(renderer_channel: Sender<RendererMessage>) -> Self {
        let input_fields = vec![InputField::new("Value")];
        let form_state = FormState::new(input_fields);
        Self {
            renderer_channel,
            form_state,
        }
    }
}

impl Component for SignalValueSetter {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let form = Form::default();
        f.render_stateful_widget(form, rect, &mut self.form_state);
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> HandleResult {
        match key_event.code {
            KeyCode::Char(ch) => self.form_state.put(ch),
            KeyCode::Backspace => self.form_state.backspace(),
            KeyCode::Up => self.form_state.up(),
            KeyCode::Down => self.form_state.down(),
            KeyCode::Left => self.form_state.left(),
            KeyCode::Right => self.form_state.right(),
            KeyCode::Enter => return HandleResult::ReleaseFocus,
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
