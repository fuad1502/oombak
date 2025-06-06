use ratatui::layout::{Constraint, Layout, Rect};

pub fn get_popup_area_bottom_right(rect: Rect) -> Rect {
    let min_width = 40;
    let min_height = 13;
    let top_margin = 3.max(rect.height as i64 - min_height - 3);
    let left_margin = 6.max(rect.width as i64 - min_width - 6);
    get_popup_area(rect, top_margin as u16, 6, 3, left_margin as u16)
}

pub fn get_popup_area_centered(rect: Rect, vert_margin: u16, hor_margin: u16) -> Rect {
    get_popup_area(rect, vert_margin, hor_margin, vert_margin, hor_margin)
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
