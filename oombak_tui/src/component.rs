use std::sync::{Arc, RwLock};

use crossterm::event::{Event, KeyEvent};
use ratatui::{layout::Rect, widgets::Block, Frame};

pub trait Component: Send + Sync {
    fn render(&mut self, f: &mut Frame, rect: Rect);

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> HandleResult;

    fn handle_resize_event(&mut self, columns: u16, rows: u16) -> HandleResult;

    fn handle_focus_gained(&mut self);

    fn get_focused_child(&self) -> Option<Arc<RwLock<dyn Component>>>;

    fn render_with_block(&mut self, f: &mut Frame, rect: Rect, block: Block) {
        let inner = block.inner(rect);
        self.render(f, inner);
        f.render_widget(block, rect);
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
