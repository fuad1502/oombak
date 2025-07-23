use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::Text,
    widgets::{Block, Clear, List, ListState, StatefulWidget, Widget},
};

use crate::styles::{dropdown::ITEM_DEFAULT_STYLE, global::SELECTED_ITEM_STYLE};

#[derive(Default)]
pub struct DropDown<'a> {
    block: Block<'a>,
}

pub struct DropDownState {
    items: Vec<String>,
    is_opened: bool,
    list_state: ListState,
}

impl<'a> DropDown<'a> {
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = block;
        self
    }
}

impl<'a> StatefulWidget for DropDown<'a> {
    type State = DropDownState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if state.is_opened {
            let viewport_height = buf.area().height - area.y;
            let mut list_area = area;
            list_area.height = usize::min(state.items.len() + 2, viewport_height as usize) as u16;
            let inner_list_area = self.block.inner(list_area);
            let list = List::from_iter(state.items.clone())
                .style(ITEM_DEFAULT_STYLE)
                .highlight_style(SELECTED_ITEM_STYLE);
            Widget::render(Clear, list_area, buf);
            Widget::render(self.block, list_area, buf);
            StatefulWidget::render(list, inner_list_area, buf, &mut state.list_state);
        } else {
            let inner_area = self.block.inner(area);
            let areas = Layout::horizontal(vec![Constraint::Min(0), Constraint::Length(2)])
                .split(inner_area);
            let item = state.selected().unwrap_or_default();
            let dropdown_symbol = Text::from("▼ ").style(ITEM_DEFAULT_STYLE);
            let text = Text::from(item).style(ITEM_DEFAULT_STYLE);
            Widget::render(self.block, area, buf);
            Widget::render(text, areas[0], buf);
            Widget::render(dropdown_symbol, areas[1], buf);
        }
    }
}

impl DropDownState {
    pub fn new(items: Vec<String>) -> Self {
        let list_state = ListState::default();
        let is_opened = false;
        DropDownState {
            items,
            is_opened,
            list_state,
        }
    }

    pub fn selected(&self) -> Option<&str> {
        if !self.items.is_empty() {
            return Some(
                self.items
                    .get(self.list_state.selected().unwrap_or_default())
                    .unwrap(),
            );
        }
        None
    }

    pub fn is_opened(&self) -> bool {
        self.is_opened
    }

    pub fn open(&mut self) {
        self.is_opened = true;
        if self.list_state.selected().is_none() {
            self.list_state.select_first();
        }
    }

    pub fn close(&mut self) {
        self.is_opened = false;
    }

    pub fn down(&mut self) {
        self.list_state.select_next();
    }

    pub fn up(&mut self) {
        self.list_state.select_previous();
    }

    pub fn select(&mut self, idx: usize) -> Result<(), &'static str> {
        if idx >= self.items.len() {
            return Err("Index out of range");
        }
        self.list_state.select(Some(idx));
        Ok(())
    }
}
