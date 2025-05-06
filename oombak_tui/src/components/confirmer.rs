use std::{
    collections::HashMap,
    sync::{mpsc::Sender, Arc, RwLock},
};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    widgets::{Paragraph, Wrap},
    Frame,
};

use crate::{
    component::{Component, HandleResult},
    threads::RendererMessage,
    widgets::{ConfirmationBox, ConfirmationState, KeyDesc, KeyId, KeyMaps},
};

pub struct Confirmer {
    message_tx: Sender<RendererMessage>,
    key_maps: KeyMaps,
    text: String,
    highlighted_state: ConfirmationState,
    selected_state: ConfirmationState,
}

impl Confirmer {
    pub fn new(message_tx: Sender<RendererMessage>) -> Self {
        let key_maps = Self::create_key_maps();
        Self {
            message_tx,
            key_maps,
            text: "".to_string(),
            highlighted_state: ConfirmationState::confirm(),
            selected_state: ConfirmationState::dismiss(),
        }
    }

    pub fn create_key_maps() -> KeyMaps {
        KeyMaps::from(HashMap::from([
            (
                KeyId::from(KeyCode::Char('h')),
                KeyDesc::from("move selection"),
            ),
            (KeyId::from(KeyCode::Left), KeyDesc::from("move selection")),
            (
                KeyId::from(KeyCode::Char('l')),
                KeyDesc::from("move selection"),
            ),
            (KeyId::from(KeyCode::Right), KeyDesc::from("move selection")),
            (
                KeyId::from(KeyCode::Enter),
                KeyDesc::from("confirm selection"),
            ),
        ]))
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    pub fn selected_state(&self) -> &ConfirmationState {
        &self.selected_state
    }

    fn notify_render(&self) {
        self.message_tx.send(RendererMessage::Render).unwrap();
    }
}

impl Component for Confirmer {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let text = Paragraph::new(&self.text[..]).wrap(Wrap { trim: false });
        let confirm_box = ConfirmationBox::new(&text);
        f.render_stateful_widget(confirm_box, rect, &mut self.highlighted_state);
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> HandleResult {
        match key_event.code {
            KeyCode::Left | KeyCode::Char('h') => {
                self.highlighted_state.set_confirm();
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.highlighted_state.set_dismiss();
            }
            KeyCode::Enter => {
                self.notify_render();
                self.selected_state = self.highlighted_state;
                return HandleResult::ReleaseFocus;
            }
            _ => (),
        }
        self.notify_render();
        HandleResult::Handled
    }

    fn handle_resize_event(&mut self, _columns: u16, _rows: u16) -> HandleResult {
        HandleResult::NotHandled
    }

    fn handle_focus_gained(&mut self) -> HandleResult {
        HandleResult::Handled
    }

    fn get_focused_child(&self) -> Option<Arc<RwLock<dyn Component>>> {
        None
    }

    fn get_key_mappings(&self) -> KeyMaps {
        self.key_maps.clone()
    }
}
