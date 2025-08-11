use std::{
    collections::HashMap,
    env, fs,
    io::BufRead,
    path::{Path, PathBuf},
    sync::mpsc::Sender,
};

use crossterm::event::{KeyCode, KeyEvent};
use file_type::FileType;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style, Styled, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, List, ListItem, ListState},
    Frame,
};

use crate::{
    component::{Component, HandleResult},
    styles::{
        file_explorer::{
            DIRECTORY_FILE_TYPE_STYLE, DIR_ITEM_STYLE, ERROR_FILE_TYPE_STYLE, FILE_ITEM_STYLE,
            FILE_PREVIEW_TAB_WIDTH, OTHER_DETAILED_TYPE_STYLE, SYSTEM_VERILOG_DETAILED_TYPE_STYLE,
        },
        global::SELECTED_ITEM_STYLE,
    },
    threads::RendererMessage,
    widgets::{KeyDesc, KeyId, KeyMaps},
};

use super::TokioSender;

pub struct FileExplorer {
    message_tx: Sender<RendererMessage>,
    request_tx: TokioSender<oombak_sim::Message>,
    path: PathBuf,
    entries: Vec<PathBuf>,
    selected_idx: Option<usize>,
    list_state: ListState,
    key_mappings: KeyMaps,
}

impl FileExplorer {
    pub fn new(
        message_tx: Sender<RendererMessage>,
        request_tx: TokioSender<oombak_sim::Message>,
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

        let main_areas =
            Layout::horizontal(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(rect);
        let right_areas =
            Layout::vertical(vec![Constraint::Length(5), Constraint::Min(0)]).split(main_areas[1]);
        let list_area = main_areas[0];
        let file_detail_area = right_areas[0];
        let file_preview_area = right_areas[1];

        let list_block = Block::bordered().border_type(BorderType::Rounded);
        let list_inner_area = list_block.inner(list_area);

        f.render_widget(list_block, list_area);
        f.render_stateful_widget(list, list_inner_area, &mut self.list_state);
        self.render_file_details(f, file_detail_area);
        self.render_file_preview(f, file_preview_area);
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
        if fs::read_dir(dir).is_ok() {
            self.path.push(dir);
            self.refresh_list();
        }
    }

    fn load_file(&self, idx: usize) {
        let mut file_path = self.path.clone();
        let file = &self.entries[idx];
        file_path.push(file);
        self.request_tx
            .blocking_send(oombak_sim::Request::load(file_path))
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

    fn render_file_details(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered().border_type(BorderType::Rounded);
        let inner_area = block.inner(area);
        let lines = self.file_detail_lines();
        frame.render_widget(block, area);
        frame.render_widget(Text::from(lines), inner_area);
    }

    fn file_detail_lines(&self) -> Vec<Line<'_>> {
        if let Some(file_path) = self.highlighted_file_path() {
            let file_name = Self::file_name_from_path(file_path);
            let (file_type, detailed_type) = Self::file_type_from_path(file_path);

            let file_name_header = Span::from("File name : ").add_modifier(Modifier::BOLD);
            let file_type_header = Span::from("File type : ").add_modifier(Modifier::BOLD);
            let file_name = Span::from(file_name);
            let file_type = if &file_type == "Directory" {
                Span::from(file_type).style(DIRECTORY_FILE_TYPE_STYLE)
            } else if &file_type == "Directory (permission denied)" {
                Span::from(file_type).style(ERROR_FILE_TYPE_STYLE)
            } else {
                Span::from(file_type)
            };

            let detailed_type_line = if let Some(detailed_type) = detailed_type {
                let detailed_type_header = Span::from("Details   : ").add_modifier(Modifier::BOLD);
                let detailed_type = if detailed_type.contains("Verilog") {
                    Span::from(detailed_type).style(SYSTEM_VERILOG_DETAILED_TYPE_STYLE)
                } else {
                    Span::from(detailed_type).style(OTHER_DETAILED_TYPE_STYLE)
                };
                Line::from(vec![detailed_type_header, detailed_type])
            } else {
                Line::from("")
            };

            vec![
                Line::from(vec![file_name_header, file_name]),
                Line::from(vec![file_type_header, file_type]),
                detailed_type_line,
            ]
        } else if let Some(0) = self.selected_idx {
            let span = Span::from("Go to parent directory")
                .style(Style::default().add_modifier(Modifier::ITALIC));
            vec![Line::from(span)]
        } else {
            vec![]
        }
    }

    fn file_name_from_path(file_path: &Path) -> String {
        file_path
            .file_name()
            .map(|s| s.to_str().unwrap_or("<invalid>"))
            .unwrap_or("<invalid>")
            .to_string()
    }

    fn file_type_from_path(file_path: &Path) -> (String, Option<String>) {
        if file_path.try_exists().is_err() {
            ("Unknown".to_string(), None)
        } else if file_path.is_dir() {
            if fs::read_dir(file_path).is_err() {
                ("Directory (permission denied)".to_string(), None)
            } else {
                ("Directory".to_string(), None)
            }
        } else if file_path.is_symlink() {
            let link_path = std::fs::read_link(file_path).unwrap();
            if file_path.exists() {
                (
                    format!("Symbolic link to: {}", link_path.to_str().unwrap()),
                    Some(Self::detailed_file_type_from_path(file_path)),
                )
            } else {
                (
                    format!("Broken symbolic link to: {}", link_path.to_str().unwrap()),
                    None,
                )
            }
        } else {
            (
                "Regular file".to_string(),
                Some(Self::detailed_file_type_from_path(file_path)),
            )
        }
    }

    fn detailed_file_type_from_path(file_path: &Path) -> String {
        if let Ok(file_type) = FileType::try_from_file(file_path) {
            file_type.name().to_string()
        } else {
            "Unknown".to_string()
        }
    }

    fn render_file_preview(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered().border_type(BorderType::Rounded);
        let inner_area = block.inner(area);
        let lines = self.file_preview_lines(inner_area.height as usize, inner_area.width as usize);
        frame.render_widget(block, area);
        frame.render_widget(Text::from(lines), inner_area);
    }

    fn file_preview_lines(
        &self,
        max_num_of_lines: usize,
        max_num_of_chars_per_line: usize,
    ) -> Vec<Line<'_>> {
        if let Some(true) = self.is_highlighted_file_text_file() {
            let file_path = self.highlighted_file_path().unwrap();
            let file = std::fs::File::open(file_path).unwrap();
            let reader = std::io::BufReader::new(file);
            reader
                .lines()
                .take(max_num_of_lines)
                .map(Result::unwrap)
                .map(format_text)
                .map(|s| truncate(s, max_num_of_chars_per_line))
                .map(Line::from)
                .collect()
        } else {
            vec![]
        }
    }

    fn is_highlighted_file_text_file(&self) -> Option<bool> {
        if let Some(file_path) = self.highlighted_file_path() {
            if file_path.is_dir() {
                return Some(false);
            }

            if FileType::try_from_file(file_path).is_err() {
                return Some(false);
            }

            Some(
                FileType::try_from_file(file_path)
                    .expect("file not found")
                    .media_types()
                    .iter()
                    .filter(|t| t.starts_with("text") || t.starts_with("application/json"))
                    .count()
                    > 0,
            )
        } else {
            None
        }
    }

    fn highlighted_file_path(&self) -> Option<&PathBuf> {
        self.selected_idx
            .filter(|i| *i > 0)
            .map(|i| &self.entries[i - 1])
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

fn format_text(text: String) -> String {
    let mut formatted_string = String::new();
    for ch in text.chars() {
        if ch.is_control() {
            if ch == '\t' {
                let current_pos = formatted_string.len();
                let num_of_spaces = FILE_PREVIEW_TAB_WIDTH - current_pos % FILE_PREVIEW_TAB_WIDTH;
                formatted_string.push_str(&" ".repeat(num_of_spaces));
            }
        } else {
            formatted_string.push(ch);
        }
    }
    formatted_string
}

fn truncate(text: String, n: usize) -> String {
    text.chars().take(n).collect()
}
