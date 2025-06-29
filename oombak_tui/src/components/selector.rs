use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Rect},
    text::Text,
    widgets::{Block, BorderType, Clear, List, ListState},
    Frame,
};
use std::{
    collections::HashMap,
    sync::{mpsc::Sender, Arc, RwLock},
};

use crate::{
    component::{Component, HandleResult},
    styles::{global::SELECTED_ITEM_STYLE, selector::DISABLED_ITEM_STYLE},
    threads::RendererMessage,
    utils,
    widgets::{KeyDesc, KeyId, KeyMaps},
};

pub struct Selector {
    selection: Vec<Selection>,
    title: String,
    list_state: ListState,
    child: Option<usize>,
    key_maps: KeyMaps,
    renderer_channel: Sender<RendererMessage>,
}

pub struct Selection {
    name: String,
    component: Arc<RwLock<dyn Component>>,
    disabled: bool,
}

impl Selector {
    pub fn new(selection: Vec<Selection>, renderer_channel: Sender<RendererMessage>) -> Self {
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

    pub fn set_selection(&mut self, selection: Vec<Selection>) {
        self.selection = selection;
    }

    pub fn enable_selection(&mut self, idx: usize) -> bool {
        if let Some(selection) = self.selection.get_mut(idx) {
            selection.disabled = false;
            true
        } else {
            false
        }
    }

    pub fn disable_selection(&mut self, idx: usize) -> bool {
        if let Some(selection) = self.selection.get_mut(idx) {
            selection.disabled = true;
            true
        } else {
            false
        }
    }

    fn create_key_maps() -> KeyMaps {
        KeyMaps::from(HashMap::from([
            (KeyId::from('q'), KeyDesc::from("Close window")),
            (KeyId::from('k'), KeyDesc::from("Move cursor up")),
            (KeyId::from(KeyCode::Up), KeyDesc::from("Move cursor up")),
            (KeyId::from('j'), KeyDesc::from("Move cursor down")),
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
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);
        let inner_area = block.inner(render_area);
        let list = List::new(self.selection.iter().map(|s| {
            if s.disabled {
                Text::from(&s.name[..]).style(DISABLED_ITEM_STYLE)
            } else {
                Text::from(&s.name[..])
            }
        }))
        .highlight_style(SELECTED_ITEM_STYLE);
        f.render_widget(Clear, render_area);
        f.render_widget(block, render_area);
        f.render_stateful_widget(list, inner_area, &mut self.list_state);
    }

    fn try_select_default_item(&mut self) {
        if self.list_state.selected().is_none() {
            self.list_state.select_first();
        }
    }

    fn get_render_area(&self, rect: Rect) -> Rect {
        let height = u16::min(rect.height, self.selection.len() as u16 + 2);
        let width = u16::min(rect.width, self.max_content_width() as u16 + 2);
        utils::layout::get_popup_area_centered(rect, width, height)
    }

    fn max_content_width(&self) -> usize {
        self.title.len().max(self.max_selection_text_width()) + 10
    }

    fn max_selection_text_width(&self) -> usize {
        self.selection
            .iter()
            .map(|s| s.name.len())
            .max()
            .unwrap_or(0)
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
            KeyCode::Up | KeyCode::Char('k') => self.list_state.select_previous(),
            KeyCode::Down | KeyCode::Char('j') => self.list_state.select_next(),
            KeyCode::Enter => {
                self.child = self
                    .list_state
                    .selected()
                    .filter(|i| !self.selection[*i].disabled);
            }
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
        self.child.map(|i| self.selection[i].component.clone())
    }

    fn get_key_mappings(&self) -> KeyMaps {
        match self.get_focused_child() {
            Some(component) => component.read().unwrap().get_key_mappings(),
            None => self.key_maps.clone(),
        }
    }
}

impl Selection {
    pub fn new(name: &str, component: Arc<RwLock<dyn Component>>) -> Self {
        Self {
            name: name.to_string(),
            component,
            disabled: true,
        }
    }
}
