use crate::widgets::{CommandKeysHelpWindow, KeyMaps, ScrollState};

pub struct KeyMapsViewer {
    page_state: ScrollState,
    key_maps: KeyMaps,
}

impl KeyMapsViewer {
    pub fn new(key_maps: KeyMaps) -> Self {
        Self {
            page_state: ScrollState::default(),
            key_maps,
        }
    }

    pub fn set_key_maps(&mut self, key_maps: KeyMaps) {
        self.key_maps = key_maps;
    }

    pub fn next_page(&mut self) {
        self.page_state.next();
    }

    pub fn prev_page(&mut self) {
        self.page_state.prev();
    }

    pub fn render_mut(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        let key_maps_window = CommandKeysHelpWindow::new(&self.key_maps);
        f.render_stateful_widget(key_maps_window, rect, &mut self.page_state);
    }
}
