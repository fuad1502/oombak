use crossterm::event::{Event, KeyEvent};
use ratatui::{layout::Rect, widgets::Block, Frame};

pub trait Component {
    fn render(&mut self, f: &mut Frame, rect: Rect);

    fn render_with_block(&mut self, f: &mut Frame, rect: Rect, block: Block) {
        let inner = block.inner(rect);
        self.render(f, inner);
        f.render_widget(block, rect);
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent);

    fn handle_event(&mut self, event: &Event) {
        if let Event::Key(key_event) = event {
            self.handle_key_event(key_event);
        }
    }
}
