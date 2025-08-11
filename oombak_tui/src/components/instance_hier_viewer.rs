use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock, RwLockReadGuard};

use crossterm::event::KeyCode;
use oombak_sim::request::ProbePointsModification;
use oombak_sim::response::LoadedDut;
use oombak_sim::{InstanceNode, Signal};
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::{Span, Text};
use ratatui::widgets::{Block, BorderType, Clear};
use ratatui::Frame;
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    text::Line,
    widgets::{List, ListItem, ListState, Paragraph},
};

use crate::styles::global::SELECTED_ITEM_STYLE;
use crate::styles::instance_hier_viewer::{INSTANCE_ITEM_STYLE, SIGNAL_ITEM_STYLE};
use crate::widgets::{KeyDesc, KeyId, KeyMaps};
use crate::{
    component::{Component, HandleResult},
    threads::RendererMessage,
};

use super::{Confirmer, TokioSender};

pub struct InstanceHierViewer {
    message_tx: Sender<RendererMessage>,
    request_tx: TokioSender<oombak_sim::Message>,
    focused_child: Option<Child>,
    confirmer: Arc<RwLock<Confirmer>>,
    root_node: Option<Arc<RwLock<InstanceHierNode>>>,
    probed_points: HashSet<String>,
    items_in_list: Vec<HierItem>,
    list_state: ListState,
    selected_item_idx: Option<usize>,
    signals_marked_to_add: HashSet<String>,
    signals_marked_to_remove: HashSet<String>,
    key_mappings: KeyMaps,
}

enum Child {
    Confirmer,
}

struct InstanceHierNode {
    path: String,
    module_name: String,
    children: Vec<Arc<RwLock<InstanceHierNode>>>,
    leafs: Vec<Arc<RwLock<InstanceHierLeaf>>>,
    is_expanded: bool,
}

#[derive(Clone)]
struct InstanceHierLeaf {
    path: String,
    module_name: String,
    signal: Signal,
    is_added: bool,
    marker: Marker,
}

#[derive(Clone)]
enum Marker {
    NotMarked,
    MarkedForAdd,
    MarkedForRemove,
}

enum HierItem {
    Instance(Arc<RwLock<InstanceHierNode>>),
    Signal(Arc<RwLock<InstanceHierLeaf>>),
}

impl InstanceHierViewer {
    pub fn new(
        message_tx: Sender<RendererMessage>,
        request_tx: TokioSender<oombak_sim::Message>,
    ) -> Self {
        let key_mappings = Self::create_key_mappings();
        let confirmer = Arc::new(RwLock::new(Confirmer::new(message_tx.clone())));
        Self {
            message_tx,
            request_tx,
            focused_child: None,
            confirmer,
            root_node: None,
            items_in_list: vec![],
            list_state: ListState::default(),
            selected_item_idx: None,
            probed_points: HashSet::default(),
            signals_marked_to_add: HashSet::default(),
            signals_marked_to_remove: HashSet::default(),
            key_mappings,
        }
    }

    pub fn set_loaded_dut(&mut self, loaded_dut: &LoadedDut) {
        self.probed_points = HashSet::from_iter(loaded_dut.probed_points.iter().cloned());
        self.root_node = Some(Arc::new(RwLock::new(InstanceHierNode::new(
            &loaded_dut.root_node,
            "",
            &self.probed_points,
        ))));
        self.selected_item_idx = Some(0);
        self.list_state.select_first();
    }

    fn create_key_mappings() -> KeyMaps {
        HashMap::from([
            (KeyId::from('q'), KeyDesc::from("confirm / dismiss changes")),
            (
                KeyId::from(KeyCode::Enter),
                KeyDesc::from("add / remove signal from probing"),
            ),
            (KeyId::from(KeyCode::Up), KeyDesc::from("scroll up")),
            (KeyId::from('k'), KeyDesc::from("scroll up")),
            (KeyId::from(KeyCode::Down), KeyDesc::from("scroll down")),
            (KeyId::from('j'), KeyDesc::from("scroll down")),
        ])
        .into()
    }
}

impl Component for InstanceHierViewer {
    fn render(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        if let Some(node) = &self.root_node {
            let (list_items, items_in_list) = Self::get_flattened_hierarchy(node);
            self.items_in_list = items_in_list;
            let list = List::new(list_items).highlight_style(SELECTED_ITEM_STYLE);
            let block = Block::bordered().border_type(BorderType::Rounded);

            let main_areas =
                Layout::horizontal(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(rect);
            let item_detail_area =
                Layout::vertical(vec![Constraint::Length(9), Constraint::Min(0)])
                    .split(main_areas[1])[0];
            let list_area = main_areas[0];
            let inner_list_area = block.inner(list_area);

            f.render_widget(block, list_area);
            f.render_stateful_widget(list, inner_list_area, &mut self.list_state);
            self.render_item_detail(f, item_detail_area);
        } else {
            let rect = Layout::vertical(vec![
                Constraint::Percentage(50),
                Constraint::Length(1),
                Constraint::Percentage(50),
            ])
            .split(rect)[1];
            let message = Paragraph::new("DUT not loaded").alignment(Alignment::Center);
            f.render_widget(message, rect);
        }

        if matches!(self.focused_child, Some(Child::Confirmer)) {
            self.render_confirmation_box(f, rect);
        }
    }

    fn handle_key_event(
        &mut self,
        key_event: &crossterm::event::KeyEvent,
    ) -> crate::component::HandleResult {
        match key_event.code {
            KeyCode::Char('q') => {
                if self.root_node.is_some() && !self.signals_marked_to_add.is_empty()
                    || !self.signals_marked_to_remove.is_empty()
                {
                    self.focused_child = Some(Child::Confirmer);
                    let mut text =
                        String::from("Would you like to apply the following changes?\n\n");
                    for s in &self.signals_marked_to_add {
                        text += &format!("(+) {s}\n");
                    }
                    for s in &self.signals_marked_to_remove {
                        text += &format!("(-) {s}\n");
                    }
                    self.confirmer.write().unwrap().set_text(&text);
                } else {
                    self.notify_render();
                    return HandleResult::ReleaseFocus;
                }
            }
            KeyCode::Enter => self.perform_action_on_selected(),
            KeyCode::Down | KeyCode::Char('j') => self.scroll_down(),
            KeyCode::Up | KeyCode::Char('k') => self.scroll_up(),
            KeyCode::F(_) => return HandleResult::NotHandled,
            _ => (),
        }
        self.notify_render();
        HandleResult::Handled
    }

    fn handle_resize_event(&mut self, _columns: u16, _rows: u16) -> HandleResult {
        self.notify_render();
        HandleResult::Handled
    }

    fn handle_focus_gained(&mut self) -> HandleResult {
        self.focused_child = None;
        let is_confirm = self.confirmer.read().unwrap().selected_state().is_confirm();
        if is_confirm.unwrap_or(false) {
            self.request_modify_probe_points();
            self.clear_marked_signals();
        }
        self.notify_render();
        HandleResult::ReleaseFocus
    }

    fn get_focused_child(&self) -> Option<Arc<RwLock<dyn Component>>> {
        match &self.focused_child {
            Some(Child::Confirmer) => Some(self.confirmer.clone()),
            None => None,
        }
    }

    fn get_key_mappings(&self) -> KeyMaps {
        match self.focused_child {
            Some(Child::Confirmer) => self.confirmer.read().unwrap().get_key_mappings(),
            None => self.key_mappings.clone(),
        }
    }
}

impl InstanceHierViewer {
    fn notify_render(&self) {
        self.message_tx.send(RendererMessage::Render).unwrap();
    }

    fn render_confirmation_box(&mut self, f: &mut Frame, rect: Rect) {
        let popup_area = Self::get_popup_area_centered(rect, 3, 6, 30, 8);
        f.render_widget(Clear, popup_area);
        self.confirmer.write().unwrap().render(f, popup_area);
    }

    fn get_popup_area_centered(
        rect: Rect,
        vert_margin: u16,
        hor_margin: u16,
        max_width: u16,
        max_height: u16,
    ) -> Rect {
        let vert_margin = (vert_margin as i64 * 2)
            .max(rect.height as i64 - max_height as i64 - vert_margin as i64)
            / 2;
        let hor_margin = (hor_margin as i64 * 2)
            .max(rect.width as i64 - max_width as i64 - hor_margin as i64)
            / 2;
        Self::get_popup_area(
            rect,
            vert_margin as u16,
            hor_margin as u16,
            vert_margin as u16,
            hor_margin as u16,
        )
    }

    fn get_popup_area(
        rect: Rect,
        top_margin: u16,
        right_margin: u16,
        bottom_margin: u16,
        left_margin: u16,
    ) -> Rect {
        let chunks = Layout::vertical(vec![
            Constraint::Length(top_margin),
            Constraint::Min(0),
            Constraint::Length(bottom_margin),
        ])
        .split(rect);
        let chunks = Layout::horizontal(vec![
            Constraint::Length(left_margin),
            Constraint::Min(0),
            Constraint::Length(right_margin),
        ])
        .split(chunks[1]);
        chunks[1]
    }

    fn get_flattened_hierarchy(
        node: &Arc<RwLock<InstanceHierNode>>,
    ) -> (Vec<ListItem<'static>>, Vec<HierItem>) {
        let mut list_items = vec![];
        let mut items_in_list = vec![];
        Self::traverse_hier_node(node, &mut list_items, &mut items_in_list, 0);
        (list_items, items_in_list)
    }

    fn traverse_hier_node(
        node: &Arc<RwLock<InstanceHierNode>>,
        list_items: &mut Vec<ListItem>,
        items_in_list: &mut Vec<HierItem>,
        depth: usize,
    ) {
        list_items.push(Self::new_instance_list_item(node, depth));
        items_in_list.push(HierItem::Instance(node.clone()));
        let node = node.read().unwrap();
        if node.is_expanded {
            for leaf in node.leafs.iter() {
                list_items.push(Self::new_signal_list_item(leaf, depth + 1));
                items_in_list.push(HierItem::Signal(leaf.clone()));
            }
            for node in node.children.iter() {
                Self::traverse_hier_node(node, list_items, items_in_list, depth + 1);
            }
        }
    }

    fn new_instance_list_item<'a>(
        node: &Arc<RwLock<InstanceHierNode>>,
        depth: usize,
    ) -> ListItem<'a> {
        let node = node.read().unwrap();
        let indentation = " ".repeat(depth * 2);
        let expand_or_collapse_symbol = if node.is_expanded { "[-]" } else { "[+]" };
        let line = Line::raw(format!(
            "{}{} {} ({})",
            indentation, expand_or_collapse_symbol, node.path, node.module_name
        ))
        .style(INSTANCE_ITEM_STYLE);
        ListItem::new(line)
    }

    fn new_signal_list_item<'a>(
        leaf: &Arc<RwLock<InstanceHierLeaf>>,
        depth: usize,
    ) -> ListItem<'a> {
        let leaf = leaf.read().unwrap();
        let indentation = " ".repeat(depth * 2);
        let added_symbol = if leaf.is_added { " (*)" } else { "" };
        let marker_symbol = match leaf.marker {
            Marker::NotMarked => "",
            Marker::MarkedForAdd => " (+)",
            Marker::MarkedForRemove => " (-)",
        };
        let line = Line::raw(format!(
            "{}{}{}{}",
            indentation, leaf.signal.name, added_symbol, marker_symbol
        ))
        .style(SIGNAL_ITEM_STYLE);
        ListItem::new(line)
    }

    fn perform_action_on_selected(&mut self) {
        let mut signals_marked_to_add = self.signals_marked_to_add.clone();
        let mut signals_marked_to_remove = self.signals_marked_to_remove.clone();
        if let Some(item) = self.get_selected_item() {
            match item {
                HierItem::Instance(node) => {
                    let mut node = node.write().unwrap();
                    node.is_expanded = !node.is_expanded;
                }
                HierItem::Signal(leaf) => {
                    let mut leaf = leaf.write().unwrap();
                    leaf.marker = match leaf.marker {
                        Marker::NotMarked if leaf.is_added => {
                            signals_marked_to_remove.insert(leaf.path.clone());
                            Marker::MarkedForRemove
                        }
                        Marker::NotMarked => {
                            signals_marked_to_add.insert(leaf.path.clone());
                            Marker::MarkedForAdd
                        }
                        Marker::MarkedForAdd => {
                            signals_marked_to_add.remove(&leaf.path);
                            Marker::NotMarked
                        }
                        Marker::MarkedForRemove => {
                            signals_marked_to_remove.remove(&leaf.path);
                            Marker::NotMarked
                        }
                    };
                }
            }
            self.notify_render();
        }
        self.signals_marked_to_add = signals_marked_to_add;
        self.signals_marked_to_remove = signals_marked_to_remove;
    }

    fn get_selected_item(&self) -> Option<&HierItem> {
        if let Some(idx) = self.selected_item_idx {
            Some(&self.items_in_list[idx])
        } else {
            None
        }
    }

    fn request_modify_probe_points(&self) {
        let probe_points_modifications = ProbePointsModification {
            to_add: self.signals_marked_to_add.clone().into_iter().collect(),
            to_remove: self.signals_marked_to_remove.clone().into_iter().collect(),
        };
        self.request_tx
            .blocking_send(oombak_sim::Request::modify_probe_points(
                probe_points_modifications,
            ))
            .unwrap();
    }

    fn clear_marked_signals(&mut self) {
        self.signals_marked_to_add.clear();
        self.signals_marked_to_remove.clear();
    }

    fn scroll_down(&mut self) {
        if let Some(idx) = self.selected_item_idx {
            self.list_state.select_next();
            let new_idx = usize::saturating_add(idx, 1);
            self.selected_item_idx = Some(usize::min(self.items_in_list.len() - 1, new_idx));
        }
    }

    fn scroll_up(&mut self) {
        if let Some(idx) = self.selected_item_idx {
            self.list_state.select_previous();
            self.selected_item_idx = Some(usize::saturating_sub(idx, 1));
        }
    }

    fn render_item_detail(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered().border_type(BorderType::Rounded);
        let inner_area = block.inner(area);
        let lines = self.item_detail_lines();
        frame.render_widget(block, area);
        frame.render_widget(Text::from(lines), inner_area);
    }

    fn item_detail_lines(&self) -> Vec<Line<'_>> {
        if let Some(item) = self.get_selected_item() {
            match item {
                HierItem::Instance(node) => Self::instance_detail_lines(node.read().unwrap()),
                HierItem::Signal(leaf) => Self::signal_detail_lines(leaf.read().unwrap()),
            }
        } else {
            vec![]
        }
    }

    fn instance_detail_lines(hier_node: RwLockReadGuard<InstanceHierNode>) -> Vec<Line> {
        let type_name = Span::from("module").magenta();
        let instance_name = Span::from(hier_node.instance_name());
        let module_name = Span::from(hier_node.module_name.clone());
        let instance_path = Span::from(hier_node.instance_path());

        let type_name_header = Span::from("Item type     : ").bold();
        let instance_name_header = Span::from("Name          : ").bold();
        let module_name_header = Span::from("Module name   : ").bold();
        let instance_path_header = Span::from("Instance path : ").bold();

        let type_line = Line::from(vec![type_name_header, type_name]);
        let instance_name_line = Line::from(vec![instance_name_header, instance_name]);
        let module_name_line = Line::from(vec![module_name_header, module_name]);
        let instance_path_line = Line::from(vec![instance_path_header, instance_path]);

        vec![
            type_line,
            instance_name_line,
            module_name_line,
            instance_path_line,
        ]
    }

    fn signal_detail_lines(hier_leaf: RwLockReadGuard<InstanceHierLeaf>) -> Vec<Line> {
        let type_name = Span::from("signal").cyan();
        let signal_name = Span::from(hier_leaf.signal.name.clone());
        let module_name = Span::from(hier_leaf.module_name.clone());
        let instance_path = Span::from(hier_leaf.instance_path().clone());
        let signal_type = Span::from(hier_leaf.signal.signal_type.to_string());
        let signal_bit_width = Span::from(hier_leaf.signal.bit_width().to_string());
        let is_added = match hier_leaf.is_added {
            true => Span::from("Yes").light_green(),
            false => Span::from("No").light_red(),
        };
        let is_marked = match hier_leaf.marker {
            Marker::NotMarked => Span::from(""),
            Marker::MarkedForAdd => Span::from(" (marked for add)").light_yellow(),
            Marker::MarkedForRemove => Span::from(" (marked for remove)").light_yellow(),
        };

        let type_name_header = Span::from("Item type     : ").bold();
        let signal_name_header = Span::from("Name          : ").bold();
        let module_name_header = Span::from("Module name   : ").bold();
        let instance_path_header = Span::from("Instance path : ").bold();
        let signal_type_header = Span::from("Type          : ").bold();
        let signal_bit_width_header = Span::from("Bit width     : ").bold();
        let is_added_header = Span::from("Is added      : ").bold();

        let type_line = Line::from(vec![type_name_header, type_name]);
        let module_name_line = Line::from(vec![module_name_header, module_name]);
        let instance_path_line = Line::from(vec![instance_path_header, instance_path]);
        let signal_name_line = Line::from(vec![signal_name_header, signal_name]);
        let signal_type_line = Line::from(vec![signal_type_header, signal_type]);
        let signal_bit_width_line = Line::from(vec![signal_bit_width_header, signal_bit_width]);
        let is_added_line = Line::from(vec![is_added_header, is_added, is_marked]);

        vec![
            type_line,
            module_name_line,
            instance_path_line,
            signal_name_line,
            signal_type_line,
            signal_bit_width_line,
            is_added_line,
        ]
    }
}

impl InstanceHierNode {
    fn new(
        instance_node: &InstanceNode,
        parent_path: &str,
        probed_points: &HashSet<String>,
    ) -> Self {
        let path = if parent_path.is_empty() {
            instance_node.name.to_string()
        } else {
            format!("{parent_path}.{}", instance_node.name)
        };
        let children: Vec<Arc<RwLock<InstanceHierNode>>> = instance_node
            .children
            .iter()
            .map(|n| InstanceHierNode::new(n, &path, probed_points))
            .map(RwLock::new)
            .map(Arc::new)
            .collect();
        let leafs = instance_node
            .signals
            .iter()
            .map(|s| InstanceHierLeaf::new(s, &path, &instance_node.module_name, probed_points))
            .map(RwLock::new)
            .map(Arc::new)
            .collect();
        InstanceHierNode {
            path,
            module_name: instance_node.module_name.clone(),
            leafs,
            is_expanded: false,
            children,
        }
    }

    fn instance_name(&self) -> String {
        self.path.split(".").last().unwrap().to_string()
    }

    fn instance_path(&self) -> String {
        let mut path: Vec<&str> = self.path.split(".").collect();
        path.pop();

        let mut path = path.join("/");
        path.insert(0, '/');
        path
    }
}

impl InstanceHierLeaf {
    fn new(
        signal: &Signal,
        parent_path: &str,
        module_name: &str,
        probed_points: &HashSet<String>,
    ) -> Self {
        let path = if parent_path.is_empty() {
            signal.name.to_string()
        } else {
            format!("{parent_path}.{}", signal.name)
        };
        let is_added = probed_points.contains(&path);
        InstanceHierLeaf {
            path,
            module_name: module_name.to_string(),
            signal: signal.clone(),
            is_added,
            marker: Marker::NotMarked,
        }
    }

    fn instance_path(&self) -> String {
        let mut path: Vec<&str> = self.path.split(".").collect();
        path.pop();

        let mut path = path.join("/");
        path.insert(0, '/');
        path
    }
}
