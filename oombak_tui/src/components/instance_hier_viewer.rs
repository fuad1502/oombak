use std::sync::mpsc::Sender;

use crossterm::event::KeyCode;
use ratatui::{
    style::{palette::tailwind::SLATE, Modifier, Style, Stylize},
    text::Line,
    widgets::{List, ListItem, ListState},
};

use crate::{
    component::{Component, HandleResult},
    render::Message,
};

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

pub struct InstanceHierViewer {
    message_tx: Sender<Message>,
    flattened_hierarchy: Vec<FlatHierItem>,
    list_state: ListState,
}

#[derive(Clone)]
struct FlatHierItem {
    name: String,
    depth: usize,
    item_type: ItemType,
}

#[derive(Clone)]
enum ItemType {
    Signal(bool),
    Instance(bool),
}

impl InstanceHierViewer {
    pub fn new(message_tx: Sender<Message>) -> Self {
        let flattened_hierarchy = vec![
            FlatHierItem {
                name: "root".to_string(),
                depth: 0,
                item_type: ItemType::Instance(true),
            },
            FlatHierItem {
                name: "sig_0".to_string(),
                depth: 1,
                item_type: ItemType::Signal(false),
            },
            FlatHierItem {
                name: "child_0".to_string(),
                depth: 1,
                item_type: ItemType::Instance(true),
            },
            FlatHierItem {
                name: "sig_1".to_string(),
                depth: 2,
                item_type: ItemType::Signal(true),
            },
        ];
        Self {
            message_tx,
            flattened_hierarchy,
            list_state: ListState::default().with_selected(Some(0)),
        }
    }

    fn notify_render(&self) {
        self.message_tx.send(Message::Render).unwrap();
    }
}

impl Component for InstanceHierViewer {
    fn render_mut(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        let list = List::new(self.flattened_hierarchy.clone()).highlight_style(SELECTED_STYLE);
        f.render_stateful_widget(list, rect, &mut self.list_state);
    }

    fn handle_key_event(
        &mut self,
        key_event: &crossterm::event::KeyEvent,
    ) -> crate::component::HandleResult {
        match key_event.code {
            KeyCode::Char('q') => return HandleResult::ReleaseFocus,
            KeyCode::Down => self.list_state.select_next(),
            KeyCode::Up => self.list_state.select_previous(),
            _ => (),
        }
        self.notify_render();
        HandleResult::Handled
    }

    fn try_propagate_event(
        &mut self,
        _event: &crossterm::event::Event,
    ) -> crate::component::HandleResult {
        HandleResult::NotHandled
    }

    fn set_focus_to_self(&mut self) {}

    fn render(&self, _f: &mut ratatui::Frame, _rect: ratatui::prelude::Rect) {
        todo!()
    }
}

impl From<FlatHierItem> for ListItem<'_> {
    fn from(item: FlatHierItem) -> Self {
        let indent = " ".repeat(item.depth * 2);
        match item.item_type {
            ItemType::Signal(included) => {
                let marker = if included { "(added)" } else { "" };
                Line::raw(format!("{indent} {} {}", item.name, marker))
                    .yellow()
                    .into()
            }
            ItemType::Instance(expanded) => {
                let expand_or_collapse = if expanded { "[-]" } else { "[+]" };
                Line::raw(format!("{indent}{} {}", expand_or_collapse, item.name)).into()
            }
        }
    }
}
