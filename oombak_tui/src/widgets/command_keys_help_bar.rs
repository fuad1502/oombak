use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Widget,
};

use super::{KeyMaps, ReversedKeyMaps};

pub struct CommandKeysHelpBar<'a> {
    key_maps: &'a KeyMaps,
    style: Style,
}

impl<'a> CommandKeysHelpBar<'a> {
    pub fn new(key_maps: &'a KeyMaps) -> Self {
        CommandKeysHelpBar {
            key_maps,
            style: Style::default(),
        }
    }
}

impl Widget for CommandKeysHelpBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let mut line = Line::default();
        let key_maps = Vec::from(&ReversedKeyMaps::from(self.key_maps));
        for key_map in &key_maps {
            let spans = Vec::from(key_map);
            if line.width() + spans_width(&spans) > area.width as usize {
                break;
            } else {
                line = append_spans(line, spans);
            }
        }

        let line = line.alignment(Alignment::Center).style(self.style);
        line.render(area, buf);
    }
}

fn spans_width(spans: &[Span]) -> usize {
    spans.iter().map(Span::width).sum()
}

fn append_spans<'a>(mut line: Line<'a>, spans: Vec<Span<'a>>) -> Line<'a> {
    for span in spans {
        line.push_span(span);
    }
    line
}

