use ratatui::{
    layout::{Constraint, Layout},
    text::{Line, Text},
    widgets::{Cell, Row, StatefulWidget, Table, Widget},
};

use super::{KeyMaps, ReversedKeyMaps, ScrollState};

pub struct CommandKeysHelpWindow<'a> {
    key_maps: &'a KeyMaps,
}

impl<'a> CommandKeysHelpWindow<'a> {
    pub fn new(key_mappings: &'a KeyMaps) -> Self {
        CommandKeysHelpWindow {
            key_maps: key_mappings,
        }
    }
}

impl StatefulWidget for CommandKeysHelpWindow<'_> {
    type State = ScrollState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let regions = Layout::vertical(vec![Constraint::Min(0), Constraint::Length(1)]).split(area);

        let table_height = regions[0].height as usize;
        let table_width = regions[0].width as usize;
        if table_height == 0 {
            return;
        };

        let mut rows: Vec<Vec<Cell>> = vec![vec![]; table_height];
        let key_maps = Vec::from(&ReversedKeyMaps::from(self.key_maps));
        let mut column_widths: Vec<usize> = vec![0; (key_maps.len() - 1) / table_height + 1];
        for (i, key_map) in key_maps.iter().enumerate() {
            let line = Line::from(key_map);
            column_widths[i / table_height] = column_widths[i / table_height].max(line.width());
            rows[i % table_height].push(Cell::from(line));
        }

        let mut fitted_columns = 0;
        let mut fitted_width = 0;
        for width in column_widths.iter() {
            if *width != 0 && fitted_width + width < table_width {
                fitted_width += width;
                fitted_columns += 1;
            }
        }
        fitted_columns = fitted_columns.max(1);
        let num_pages = (key_maps.len() - 1) / (fitted_columns * table_height) + 1;

        state.set_viewport_length(1);
        state.set_content_length(num_pages);
        let current_page = state.start_position() + 1;
        let text = Text::from(format!("Page {} of {}", current_page, num_pages));

        let start_column = (current_page - 1) * fitted_columns;
        let end_column = start_column + fitted_columns - 1;
        let rows: Vec<Row> = rows
            .iter()
            .filter(|r| !r.is_empty())
            .map(|r| Row::new(Vec::from(&r[start_column..=end_column.min(r.len() - 1)])))
            .collect();
        let constraints: Vec<Constraint> = column_widths
            .iter()
            .skip(start_column)
            .take(fitted_columns)
            .map(|x| Constraint::Length(*x as u16))
            .collect();
        let table = Table::default().widths(constraints).rows(rows);

        Widget::render(table, regions[0], buf);
        Widget::render(text, regions[1], buf);
    }
}
