use ratatui::layout::{Constraint, Layout, Rect};

pub fn get_popup_area_bottom_right(rect: Rect) -> Rect {
    let min_width = 40;
    let min_height = 13;
    let top_margin = 3.max(rect.height as i64 - min_height - 3);
    let left_margin = 6.max(rect.width as i64 - min_width - 6);
    get_popup_area(rect, top_margin as u16, 6, 3, left_margin as u16)
}

pub fn get_popup_area_centered(rect: Rect, width: u16, height: u16) -> Rect {
    let height = height.min(rect.height);
    let width = width.min(rect.width);
    let top_margin = (rect.height - height) / 2;
    let bottom_margin = rect.height - top_margin - height;
    let left_margin = (rect.width - width) / 2;
    let right_margin = rect.width - left_margin - width;
    get_popup_area(rect, top_margin, right_margin, bottom_margin, left_margin)
}

pub fn get_popup_area(
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
