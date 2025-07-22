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

pub struct SignalDisplayPropertiesSetter {
    form_state: FormState,
    renderer_channel: Sender<RendererMessage>,
}

impl SignalDisplayPropertiesSetter {
    pub fn new(renderer_channel: Sender<RendererMessage>) -> Self {
        let input_fields = vec![
            InputField::dropdown("Radix", &["Binary", "Hexadecimal", "Decimal"]),
            InputField::dropdown("Plot type", &["Digital", "Analog"]),
            InputField::text("Plot height"),
        ];
        let form_state = FormState::new(input_fields);
        Self {
            form_state,
            renderer_channel,
        }
    }
}

impl Component for SignalDisplayPropertiesSetter {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let form = Form::default();
        f.render_stateful_widget(form, rect, &mut self.form_state);
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> HandleResult {
        match key_event.code {
            KeyCode::Char(ch) if self.form_state.is_command_line() => self.form_state.put(ch),
            KeyCode::Up | KeyCode::Char('k') => self.form_state.up(),
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => self.form_state.down(),
            KeyCode::Left | KeyCode::Char('h') => self.form_state.left(),
            KeyCode::Right | KeyCode::Char('l') => self.form_state.right(),
            KeyCode::Backspace => self.form_state.backspace(),
            KeyCode::Enter => {
                if self.form_state.is_apply() || self.form_state.is_cancel() {
                    return HandleResult::ReleaseFocus;
                } else if self.form_state.is_dropdown() {
                    self.form_state.enter();
                } else {
                    self.form_state.down();
                }
            }
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
