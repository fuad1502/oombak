use ratatui::style::{Color, Modifier, Style};

pub mod global {
    use super::*;

    pub const SELECTED_ITEM_STYLE: Style =
        Style::new().bg(Color::Blue).add_modifier(Modifier::BOLD);
}

pub mod file_explorer {
    use super::*;

    pub const DIR_ITEM_STYLE: Style = Style::new().fg(Color::Green);
    pub const FILE_ITEM_STYLE: Style = Style::new();
}

pub mod terminal {
    use super::*;

    pub const COMMAND_LINE_STYLE: Style = Style::new().bg(Color::Blue);
    pub const COMMAND_LINE_HEADER_STYLE: Style = Style::new().fg(Color::Black).bg(Color::Yellow);
    pub const SUCCESS_OUTPUT_STYLE: Style = Style::new().fg(Color::Green);
    pub const FAIL_OUTPUT_STYLE: Style = Style::new().fg(Color::Red);
    pub const TEXT_CURSOR_STYLE: Style = Style::new().fg(Color::Black).bg(Color::White);
}

pub mod wave_viewer {
    use super::*;

    pub const WAVEFORM_STYLE: Style = Style::new().fg(Color::White);
    pub const TIMEBAR_STYLE: Style = Style::new().fg(Color::White);
    pub const CURSOR_STYLE: Style = Style::new().bg(Color::Red);
    pub const SELECTED_WAVEFORM_STYLE: Style = Style::new().fg(Color::Green);
}

pub mod signals_viewer {
    use super::*;

    pub const SELECTED_SIGNAL_STYLE: Style = Style::new().bg(Color::Blue);
}

pub mod instance_hier_viewer {
    use super::*;

    pub const INSTANCE_ITEM_STYLE: Style = Style::new()
        .fg(Color::White)
        .add_modifier(Modifier::UNDERLINED);
    pub const SIGNAL_ITEM_STYLE: Style = Style::new()
        .fg(Color::Yellow)
        .add_modifier(Modifier::ITALIC);
}
