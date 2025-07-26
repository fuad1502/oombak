use std::{
    collections::HashMap,
    sync::{mpsc::Sender, Arc, RwLock},
};

use bitvec::vec::BitVec;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, Frame};

use crate::{
    component::{Component, HandleResult},
    components::TokioSender,
    threads::RendererMessage,
    utils,
    widgets::{Form, FormState, InputField, KeyDesc, KeyId, KeyMaps},
};

pub struct SignalValueSetter {
    signal_name: String,
    renderer_channel: Sender<RendererMessage>,
    sim_request_channel: TokioSender<oombak_sim::Message>,
    form_state: FormState,
    key_maps: KeyMaps,
}

impl SignalValueSetter {
    pub fn new(
        signal_name: String,
        renderer_channel: Sender<RendererMessage>,
        sim_request_channel: TokioSender<oombak_sim::Message>,
    ) -> Self {
        let input_fields = vec![InputField::text("Value")];
        let form_state = FormState::new(input_fields);
        Self {
            signal_name,
            renderer_channel,
            sim_request_channel,
            form_state,
            key_maps: Self::create_key_maps(),
        }
    }

    fn create_key_maps() -> KeyMaps {
        KeyMaps::from(HashMap::from([
            (KeyId::from('q'), KeyDesc::from("close window")),
            (KeyId::from(KeyCode::Esc), KeyDesc::from("close window")),
            (KeyId::from('k'), KeyDesc::from("move up")),
            (KeyId::from(KeyCode::Up), KeyDesc::from("move up")),
            (KeyId::from('j'), KeyDesc::from("move down")),
            (KeyId::from(KeyCode::Down), KeyDesc::from("move down")),
            (KeyId::from(KeyCode::Tab), KeyDesc::from("move down")),
            (
                KeyId::from(KeyCode::Enter),
                KeyDesc::from("confirm; move down"),
            ),
        ]))
    }

    fn parse_user_input(entries: &[String]) -> Result<BitVec<u32>, String> {
        let value = utils::bitvec_str::parse(&entries[0]).map_err(|e| {
            format!(
                "Cannot convert value input ({}) to BitVec: {}",
                entries[0], e
            )
        })?;
        Ok(value)
    }

    fn request_set_signal(&self, value: BitVec<u32>) {
        let message = oombak_sim::Request::set_signal(self.signal_name.clone(), value);
        self.sim_request_channel.blocking_send(message).unwrap();
    }
}

impl Component for SignalValueSetter {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let form = Form::default();
        f.render_stateful_widget(form, rect, &mut self.form_state);
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> HandleResult {
        match key_event.code {
            KeyCode::Char(ch) if self.form_state.is_command_line() => self.form_state.put(ch),
            KeyCode::Backspace => self.form_state.backspace(),
            KeyCode::Up => self.form_state.up(),
            KeyCode::Down | KeyCode::Tab => self.form_state.down(),
            KeyCode::Left => self.form_state.left(),
            KeyCode::Right => self.form_state.right(),
            KeyCode::Enter => {
                if self.form_state.is_apply() {
                    match Self::parse_user_input(&self.form_state.entries()) {
                        Ok(value) => self.request_set_signal(value),
                        Err(_) => todo!(),
                    }
                    return HandleResult::ReleaseFocus;
                } else if self.form_state.is_cancel() {
                    return HandleResult::ReleaseFocus;
                } else {
                    self.form_state.down();
                }
            }
            KeyCode::Char('q') | KeyCode::Esc => return HandleResult::ReleaseFocus,
            KeyCode::F(_) => return HandleResult::NotHandled,
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
        match self.get_focused_child() {
            Some(child) => child.read().unwrap().get_key_mappings(),
            None => self.key_maps.clone(),
        }
    }
}
