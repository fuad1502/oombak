use std::sync::{Arc, RwLock};

use crossterm::event::{Event, KeyEvent};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    widgets::Block,
    Frame,
};

use crate::widgets::{KeyMapHelpBar, KeyMaps};

pub trait Component: Send + Sync {
    fn render(&mut self, f: &mut Frame, rect: Rect);

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> HandleResult;

    fn handle_resize_event(&mut self, columns: u16, rows: u16) -> HandleResult;

    fn handle_focus_gained(&mut self);

    fn get_focused_child(&self) -> Option<Arc<RwLock<dyn Component>>>;

    fn get_key_mappings(&self) -> KeyMaps;

    fn render_with_block(&mut self, f: &mut Frame, rect: Rect, block: Block) {
        let inner = block.inner(rect);
        self.render(f, inner);
        f.render_widget(block, rect);
    }

    fn render_with_command_keys_help_bar(&mut self, f: &mut Frame, rect: Rect) {
        let key_maps = self.get_key_mappings();
        let help_bar = KeyMapHelpBar::new(&key_maps);
        let chunks = Layout::vertical(vec![Constraint::Min(0), Constraint::Length(1)]).split(rect);
        f.render_widget(help_bar, chunks[1]);
        self.render(f, chunks[0]);
    }

    fn handle_event(&mut self, event: &Event) -> HandleResult {
        match self.try_propagate_event(event) {
            HandleResult::Handled => HandleResult::Handled,
            HandleResult::ReleaseFocus => {
                self.handle_focus_gained();
                HandleResult::Handled
            }
            HandleResult::NotHandled => match event {
                Event::Key(key_event) => self.handle_key_event(key_event),
                Event::Resize(columns, rows) => self.handle_resize_event(*columns, *rows),
                _ => HandleResult::NotHandled,
            },
        }
    }

    fn try_propagate_event(&mut self, event: &Event) -> HandleResult {
        if let Some(child) = &self.get_focused_child() {
            child.write().unwrap().handle_event(event)
        } else {
            HandleResult::NotHandled
        }
    }
}

#[derive(PartialEq)]
pub enum HandleResult {
    Handled,
    NotHandled,
    ReleaseFocus,
}
