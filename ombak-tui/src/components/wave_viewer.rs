use bitvec::vec::BitVec;
use ratatui::widgets::Block;

use crate::{component::Component, utils::bitvec_str, widgets::Waveform};

#[derive(Default)]
pub struct WaveViewer {}

impl Component for WaveViewer {
    fn render(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        let block = Block::bordered().title("Waveform");
        let option = bitvec_str::Option {
            width: 8,
            ..Default::default()
        };
        let waveform = Waveform::new(
            vec![
                BitVec::from_slice(&[0xaa]),
                BitVec::from_slice(&[0xfa]),
                BitVec::from_slice(&[0xfa]),
            ],
            1,
            10,
            option,
        )
        .block(block);
        f.render_widget(waveform, rect);
    }

    fn handle_key_event(&mut self, _key_event: &crossterm::event::KeyEvent) {
        todo!()
    }
}
