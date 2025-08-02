use ratatui::style::{Color, Modifier, Style};

pub mod global {
    use super::*;

    pub const SELECTED_ITEM_STYLE: Style =
        Style::new().bg(Color::Blue).add_modifier(Modifier::BOLD);
}

pub mod root {
    use super::*;

    pub const TITLE_STYLE: Style = Style::new().add_modifier(Modifier::BOLD);
    pub const VERSION_STYLE: Style = Style::new()
        .add_modifier(Modifier::ITALIC)
        .fg(Color::DarkGray);
}

pub mod file_explorer {
    use super::*;

    pub const DIR_ITEM_STYLE: Style = Style::new().fg(Color::Green);
    pub const FILE_ITEM_STYLE: Style = Style::new();
}

pub mod terminal {
    use super::*;

    pub const COMMAND_LINE_STYLE: Style = Style::new().bg(Color::Blue).fg(Color::Reset);
    pub const COMMAND_LINE_HEADER_STYLE: Style = Style::new().fg(Color::Black).bg(Color::Yellow);
    pub const NORMAL_OUTPUT_STYLE: Style = Style::new().fg(Color::Green);
    pub const NOTIFICATION_OUTPUT_STYLE: Style = Style::new().fg(Color::Gray);
    pub const ERROR_OUTPUT_STYLE: Style = Style::new().fg(Color::Red);
    pub const TEXT_CURSOR_STYLE: Style = Style::new().fg(Color::Black).bg(Color::White);
}

pub mod wave_viewer {
    use super::*;

    pub const WAVEFORM_STYLE: Style = Style::new().fg(Color::White);
    pub const TIMEBAR_STYLE: Style = Style::new();
    pub const CURSOR_STYLE: Style = Style::new().bg(Color::Magenta);
    pub const TIME_INDICATOR_STYLE: Style = Style::new().fg(Color::Black).bg(Color::Magenta);
    pub const SELECTED_WAVEFORM_STYLE: Style = Style::new().fg(Color::Green);
}

pub mod signals_viewer {
    use super::*;

    pub const SELECTED_SIGNAL_STYLE: Style = Style::new().bg(Color::DarkGray);
    pub const SIGNAL_NAME_STYLE: Style = Style::new().fg(Color::White).add_modifier(Modifier::BOLD);
    pub const SIGNAL_WIDTH_STYLE: Style = Style::new().fg(Color::LightGreen);
    pub const SIGNAL_VALUE_STYLE: Style = Style::new()
        .fg(Color::LightYellow)
        .add_modifier(Modifier::ITALIC);
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

pub mod command_keys_help_bar {
    use super::*;

    pub const KEY_ID_STYLE: Style = Style::new().fg(Color::Blue).add_modifier(Modifier::BOLD);
    pub const DESCRIPTION_STYLE: Style =
        Style::new().fg(Color::Green).add_modifier(Modifier::ITALIC);
}

pub mod selector {
    use super::*;

    pub const DISABLED_ITEM_STYLE: Style = Style::new().fg(Color::DarkGray);
}

pub mod form {
    use super::*;

    pub const HIGHLIGHTED_INPUT_FIELD_BORDER_STYLE: Style = Style::new().fg(Color::Green);
    pub const NORMAL_FIELD_BORDER_STYLE: Style = Style::new().fg(Color::Reset);
    pub const INPUT_FIELD_STYLE: Style = Style::new().bg(Color::Reset).fg(Color::Reset);
}

pub mod dropdown {
    use super::*;

    pub const ITEM_DEFAULT_STYLE: Style = Style::new().fg(Color::White);
}
