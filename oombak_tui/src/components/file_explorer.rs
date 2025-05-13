use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    sync::mpsc::Sender,
};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::Styled,
    widgets::{List, ListItem, ListState},
    Frame,
};

use crate::{
    component::{Component, HandleResult},
    styles::{
        file_explorer::{DIR_ITEM_STYLE, FILE_ITEM_STYLE},
        global::SELECTED_ITEM_STYLE,
    },
    threads::RendererMessage,
    widgets::{KeyDesc, KeyId, KeyMaps},
};

pub struct FileExplorer {
    message_tx: Sender<RendererMessage>,
    request_tx: Sender<oombak_sim::Request>,
    path: PathBuf,
    entries: Vec<PathBuf>,
    selected_idx: Option<usize>,
    list_state: ListState,
    key_mappings: KeyMaps,
}

impl FileExplorer {
    pub fn new(
        message_tx: Sender<RendererMessage>,
        request_tx: Sender<oombak_sim::Request>,
    ) -> Self {
        let path = env::current_dir().unwrap();
        let entries = Self::get_sorted_entries(&path);
        let key_mappings = Self::create_key_mappings();
        Self {
            message_tx,
            request_tx,
            path,
            entries,
            selected_idx: None,
            list_state: ListState::default(),
            key_mappings,
        }
    }

    fn create_key_mappings() -> KeyMaps {
        HashMap::from([
            (KeyId::from('q'), KeyDesc::from("close window")),
            (KeyId::from(KeyCode::Enter), KeyDesc::from("open")),
            (KeyId::from(KeyCode::Up), KeyDesc::from("scroll up")),
            (KeyId::from('k'), KeyDesc::from("scroll up")),
            (KeyId::from(KeyCode::Down), KeyDesc::from("scroll down")),
            (KeyId::from('j'), KeyDesc::from("scroll down")),
        ])
        .into()
    }
}

impl Component for FileExplorer {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let items = self.new_list_items();
        if self.list_state.selected().is_none() && !items.is_empty() {
            self.list_state.select_first();
            self.selected_idx = Some(0);
        }
        let list = List::new(items).highlight_style(SELECTED_ITEM_STYLE);
        f.render_stateful_widget(list, rect, &mut self.list_state);
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> HandleResult {
        match key_event.code {
            KeyCode::Up | KeyCode::Char('k') => self.handle_up_key_press(),
            KeyCode::Down | KeyCode::Char('j') => self.handle_down_key_press(),
            KeyCode::Enter => {
                if self.handle_enter_key_press() {
                    self.reset_path();
                    return HandleResult::ReleaseFocus;
                }
            }
            KeyCode::Char('q') => return HandleResult::ReleaseFocus,
            KeyCode::F(_) => return HandleResult::NotHandled,
            _ => (),
        }
        self.notify_render();
        HandleResult::Handled
    }

    fn handle_resize_event(&mut self, _columns: u16, _rows: u16) -> HandleResult {
        HandleResult::NotHandled
    }

    fn handle_focus_gained(&mut self) -> HandleResult {
        HandleResult::Handled
    }

    fn get_focused_child(&self) -> Option<std::sync::Arc<std::sync::RwLock<dyn Component>>> {
        None
    }

    fn get_key_mappings(&self) -> KeyMaps {
        self.key_mappings.clone()
    }
}

impl FileExplorer {
    fn get_sorted_entries(path: &Path) -> Vec<PathBuf> {
        let mut entries: Vec<PathBuf> = fs::read_dir(path)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        entries.sort_by(|a, b| a.file_name().unwrap().cmp(b.file_name().unwrap()));
        entries.sort_by_key(|a| std::cmp::Reverse(a.is_dir()));
        entries
    }

    fn handle_enter_key_press(&mut self) -> bool {
        if let Some(i) = self.selected_idx {
            if i == 0 {
                self.move_to_parent_dir();
            } else if self.entries[i - 1].is_dir() {
                self.move_into_dir(i - 1);
            } else {
                self.load_file(i - 1);
                return true;
            }
        }
        false
    }

    fn move_to_parent_dir(&mut self) {
        self.path.pop();
        self.refresh_list();
    }

    fn move_into_dir(&mut self, idx: usize) {
        let dir = &self.entries[idx];
        self.path.push(dir);
        self.refresh_list();
    }

    fn load_file(&self, idx: usize) {
        let mut file_path = self.path.clone();
        let file = &self.entries[idx];
        file_path.push(file);
        self.request_tx
            .send(oombak_sim::Request::load(file_path))
            .unwrap();
    }

    fn reset_path(&mut self) {
        self.path = env::current_dir().unwrap();
        self.refresh_list();
    }

    fn refresh_list(&mut self) {
        self.entries = Self::get_sorted_entries(&self.path);
        self.selected_idx = Some(0);
        self.list_state.select_first();
    }

    fn handle_down_key_press(&mut self) {
        self.list_state.select_next();
        if let Some(i) = self.selected_idx {
            if i < self.entries.len() {
                self.selected_idx = Some(i + 1);
            }
        }
    }

    fn handle_up_key_press(&mut self) {
        self.list_state.select_previous();
        if let Some(i) = self.selected_idx {
            if i > 0 {
                self.selected_idx = Some(i - 1);
            }
        }
    }

    fn new_list_items<'a>(&self) -> Vec<ListItem<'a>> {
        let mut items: Vec<ListItem> = self
            .entries
            .iter()
            .map(|p| (Self::new_list_item(p), p.is_dir()))
            .map(|(i, d)| {
                if d {
                    i.set_style(DIR_ITEM_STYLE)
                } else {
                    i.set_style(FILE_ITEM_STYLE)
                }
            })
            .collect();
        items.insert(0, ListItem::new("../").set_style(DIR_ITEM_STYLE));
        items
    }

    fn new_list_item<'a>(p: &Path) -> ListItem<'a> {
        let file_name = p.file_name().unwrap().to_string_lossy().to_string();
        let text = if p.is_dir() {
            format!("{file_name}/")
        } else {
            file_name
        };
        ListItem::new(text)
    }

    fn notify_render(&self) {
        self.message_tx.send(RendererMessage::Render).unwrap();
    }
}
