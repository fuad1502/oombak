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

pub struct PeriodicSignalSetter {
    signal_name: String,
    renderer_channel: Sender<RendererMessage>,
    sim_request_channel: TokioSender<oombak_sim::Message>,
    form_state: FormState,
    key_maps: KeyMaps,
}

impl PeriodicSignalSetter {
    pub fn new(
        signal_name: String,
        renderer_channel: Sender<RendererMessage>,
        sim_request_channel: TokioSender<oombak_sim::Message>,
    ) -> Self {
        let input_fields = vec![
            InputField::text("Period"),
            InputField::text("Low state value"),
            InputField::text("High state value"),
        ];
        let form_state = FormState::new(input_fields);
        Self {
            signal_name,
            renderer_channel,
            form_state,
            sim_request_channel,
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

    fn parse_user_input(entries: &[String]) -> Result<(usize, BitVec<u32>, BitVec<u32>), String> {
        let period = entries[0].parse::<usize>().map_err(|e| {
            format!(
                "Cannot convert period input ({}) to integer: {}",
                entries[0], e
            )
        })?;
        let low_value = utils::bitvec_str::parse(&entries[1]).map_err(|e| {
            format!(
                "Cannot convert low state value input ({}) to BitVec: {}",
                entries[1], e
            )
        })?;
        let high_value = utils::bitvec_str::parse(&entries[2]).map_err(|e| {
            format!(
                "Cannot convert high state value input ({}) to BitVec: {}",
                entries[1], e
            )
        })?;
        Ok((period, low_value, high_value))
    }

    fn request_set_periodic(&self, period: usize, low_value: BitVec<u32>, high_value: BitVec<u32>) {
        let message = oombak_sim::Request::set_periodic(
            self.signal_name.clone(),
            period,
            low_value,
            high_value,
        );
        self.sim_request_channel.blocking_send(message).unwrap();
    }
}

impl Component for PeriodicSignalSetter {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let form = Form::default();
        f.render_stateful_widget(form, rect, &mut self.form_state);
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> HandleResult {
        match key_event.code {
            KeyCode::Char(ch) if self.form_state.is_command_line() => self.form_state.put(ch),
            KeyCode::Backspace => self.form_state.backspace(),
            KeyCode::Up | KeyCode::Char('k') => self.form_state.up(),
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => self.form_state.down(),
            KeyCode::Left | KeyCode::Char('h') => self.form_state.left(),
            KeyCode::Right | KeyCode::Char('l') => self.form_state.right(),
            KeyCode::Enter => {
                if self.form_state.is_apply() {
                    match Self::parse_user_input(&self.form_state.entries()) {
                        Ok((period, low_value, high_value)) => {
                            self.request_set_periodic(period, low_value, high_value)
                        }
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
