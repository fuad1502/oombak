use crossterm::event::{Event, KeyEvent};
use ratatui::{layout::Rect, widgets::Block, Frame};

pub trait Component {
    fn render_with_block(&mut self, f: &mut Frame, rect: Rect, block: Block) {
        let inner = block.inner(rect);
        self.render(f, inner);
        f.render_widget(block, rect);
    }

    fn render(&mut self, f: &mut Frame, rect: Rect);

    fn handle_event(&mut self, event: &Event) -> bool {
        if !self.propagate_event(event) {
            self.set_focus();
            if let Event::Key(key_event) = event {
                self.handle_key_event(key_event)
            } else {
                false
            }
        } else {
            true
        }
    }

    fn propagate_event(&mut self, event: &Event) -> bool {
        if let Some(child) = self.get_focused_child() {
            child.handle_event(event)
        } else {
            false
        }
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> bool;

    fn set_focus(&mut self);

    fn get_focused_child(&mut self) -> Option<&mut dyn Component>;
}
