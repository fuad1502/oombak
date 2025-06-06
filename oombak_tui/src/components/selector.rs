use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Rect},
    widgets::{Block, Clear, List, ListState},
    Frame,
};
use std::{
    collections::HashMap,
    sync::{mpsc::Sender, Arc, RwLock},
};

use crate::{
    component::{Component, HandleResult},
    styles::global::SELECTED_ITEM_STYLE,
    threads::RendererMessage,
    utils,
    widgets::{KeyDesc, KeyId, KeyMaps},
};

pub struct Selector {
    selection: Vec<(String, Arc<RwLock<dyn Component>>)>,
    title: String,
    list_state: ListState,
    child: Option<usize>,
    key_maps: KeyMaps,
    renderer_channel: Sender<RendererMessage>,
}

impl Selector {
    pub fn new(
        selection: Vec<(String, Arc<RwLock<dyn Component>>)>,
        renderer_channel: Sender<RendererMessage>,
    ) -> Self {
        Self {
            selection,
            title: String::new(),
            list_state: ListState::default(),
            child: None,
            key_maps: Self::create_key_maps(),
            renderer_channel,
        }
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    fn create_key_maps() -> KeyMaps {
        KeyMaps::from(HashMap::from([
            (KeyId::from('q'), KeyDesc::from("Close window")),
            (KeyId::from(KeyCode::Up), KeyDesc::from("Move cursor up")),
            (
                KeyId::from(KeyCode::Down),
                KeyDesc::from("Move cursor down"),
            ),
            (
                KeyId::from(KeyCode::Enter),
                KeyDesc::from("Select highlighted"),
            ),
        ]))
    }

    fn render_self(&mut self, f: &mut Frame, rect: Rect) {
        self.try_select_default_item();
        let render_area = self.get_render_area(rect);
        let block = Block::bordered()
            .title(&self.title[..])
            .title_alignment(Alignment::Center);
        let inner_area = block.inner(render_area);
        let list =
            List::new(self.selection.iter().map(|s| &s.0[..])).highlight_style(SELECTED_ITEM_STYLE);
        f.render_widget(Clear::default(), render_area);
        f.render_widget(block, render_area);
        f.render_stateful_widget(list, inner_area, &mut self.list_state);
    }

    fn try_select_default_item(&mut self) {
        if self.list_state.selected() == None {
            self.list_state.select_first();
        }
    }

    fn get_render_area(&self, rect: Rect) -> Rect {
        let height = usize::min(rect.height as usize, self.selection.len() + 2);
        let width = usize::min(rect.width as usize, self.max_content_width() + 2);
        let vert_margin = (rect.height - height as u16) / 2;
        let hor_margin = (rect.width - width as u16) / 2;
        utils::layout::get_popup_area_centered(rect, vert_margin, hor_margin)
    }

    fn max_content_width(&self) -> usize {
        self.title.len().max(self.max_selection_text_width()) + 10
    }

    fn max_selection_text_width(&self) -> usize {
        self.selection.iter().map(|s| s.0.len()).max().unwrap_or(0)
    }
}

impl Component for Selector {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        match self.get_focused_child() {
            Some(component) => component.write().unwrap().render(f, rect),
            None => self.render_self(f, rect),
        }
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> HandleResult {
        match key_event.code {
            KeyCode::Up => self.list_state.select_previous(),
            KeyCode::Down => self.list_state.select_next(),
            KeyCode::Enter => self.child = self.list_state.selected(),
            KeyCode::Char('q') => return HandleResult::ReleaseFocus,
            _ => (),
        }
        self.renderer_channel.send(RendererMessage::Render).unwrap();
        HandleResult::Handled
    }

    fn handle_resize_event(&mut self, _: u16, _: u16) -> HandleResult {
        HandleResult::NotHandled
    }

    fn handle_focus_gained(&mut self) -> crate::component::HandleResult {
        self.child = None;
        HandleResult::ReleaseFocus
    }

    fn get_focused_child(&self) -> Option<Arc<RwLock<dyn Component>>> {
        self.child.map(|i| self.selection[i].1.clone())
    }

    fn get_key_mappings(&self) -> KeyMaps {
        match self.get_focused_child() {
            Some(component) => component.read().unwrap().get_key_mappings(),
            None => self.key_maps.clone(),
        }
    }
}
