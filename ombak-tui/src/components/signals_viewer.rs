use ratatui::widgets::Block;

use crate::component::Component;

#[derive(Default)]
pub struct SignalsViewer {}

impl Component for SignalsViewer {
    fn render(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        let block = Block::bordered().title("Signals");
        f.render_widget(block, rect);
    }

    fn handle_key_event(&mut self, _key_event: &crossterm::event::KeyEvent) {
        todo!()
    }
}
