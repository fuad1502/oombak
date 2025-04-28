use std::collections::{BTreeMap, HashMap};

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Widget,
};

use crate::styles::command_keys_help_bar::{DESCRIPTION_STYLE, KEY_ID_STYLE};

#[derive(Clone)]
pub struct KeyMaps(HashMap<KeyId, String>);

pub struct ReversedKeyMaps(BTreeMap<String, Vec<KeyId>>);

pub struct KeyMap {
    pub key_ids: Vec<KeyId>,
    pub description: String,
}

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct KeyId {
    pub key_code: KeyCode,
    pub key_modifiers: KeyModifiers,
}

pub struct KeyMapHelpBar<'a> {
    key_maps: &'a KeyMaps,
    style: Style,
}

impl<'a> KeyMapHelpBar<'a> {
    pub fn new(key_maps: &'a KeyMaps) -> Self {
        KeyMapHelpBar {
            key_maps,
            style: Style::default(),
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Widget for KeyMapHelpBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let mut line = Line::default();
        for key_map in ReversedKeyMaps::from(self.key_maps).0 {
            let key_map = KeyMap::from(key_map);
            let key_ids = Self::key_ids_to_string(&key_map.key_ids);
            let spans = vec![
                Span::from("["),
                Span::from(key_ids).style(KEY_ID_STYLE),
                Span::from(": "),
                Span::from(key_map.description).style(DESCRIPTION_STYLE),
                Span::from("] "),
            ];
            if line.width() + Self::spans_width(&spans) > area.width as usize {
                break;
            } else {
                line = Self::append_spans(line, spans);
            }
        }

        let line = line.alignment(Alignment::Center).style(self.style);
        line.render(area, buf);
    }
}

impl KeyMapHelpBar<'_> {
    fn key_ids_to_string(key_ids: &[KeyId]) -> String {
        if key_ids.len() > 1 {
            key_ids
                .iter()
                .map(KeyId::to_string)
                .collect::<Vec<String>>()
                .join(", ")
        } else if !key_ids.is_empty() {
            key_ids[0].to_string()
        } else {
            "".to_string()
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
}

impl KeyMaps {
    pub fn merge_mappings(higher_prio: &Self, lower_prio: &Self) -> Self {
        let mut merged_mappings = higher_prio.0.clone();
        for (k, v) in lower_prio.0.iter() {
            if !higher_prio.0.contains_key(k) {
                merged_mappings.insert(k.clone(), v.clone());
            }
        }
        KeyMaps(merged_mappings)
    }
}

impl From<HashMap<KeyId, String>> for KeyMaps {
    fn from(value: HashMap<KeyId, String>) -> Self {
        KeyMaps(value)
    }
}

impl From<&KeyMaps> for ReversedKeyMaps {
    fn from(key_maps: &KeyMaps) -> Self {
        let mut reversed_key_maps: BTreeMap<String, Vec<KeyId>> = BTreeMap::new();
        for (k, v) in key_maps.0.iter() {
            if let Some(key_ids) = reversed_key_maps.get_mut(v) {
                key_ids.push(k.clone());
            } else {
                reversed_key_maps.insert(v.to_string(), vec![k.clone()]);
            }
        }
        ReversedKeyMaps(reversed_key_maps)
    }
}

impl From<(String, Vec<KeyId>)> for KeyMap {
    fn from(value: (String, Vec<KeyId>)) -> Self {
        KeyMap {
            key_ids: value.1,
            description: value.0,
        }
    }
}

impl std::fmt::Display for KeyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let key = match self.key_code {
            KeyCode::Char(':') => "<colon>".to_string(),
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Up => "↑".to_string(),
            KeyCode::Down => "↓".to_string(),
            KeyCode::Left => "←".to_string(),
            KeyCode::Right => "→".to_string(),
            code => code.to_string(),
        };
        let modifiers = self
            .key_modifiers
            .iter()
            .map(|m| match m {
                KeyModifiers::CONTROL => "Ctrl",
                KeyModifiers::ALT => "Alt",
                KeyModifiers::SHIFT => "Shift",
                _ => unimplemented!(),
            })
            .collect::<Vec<&'static str>>()
            .join("+");
        if modifiers.is_empty() {
            write!(f, "{key}")
        } else {
            write!(f, "{modifiers}-{key}")
        }
    }
}

impl From<char> for KeyId {
    fn from(ch: char) -> Self {
        KeyId {
            key_code: KeyCode::Char(ch),
            key_modifiers: KeyModifiers::NONE,
        }
    }
}

impl From<KeyCode> for KeyId {
    fn from(key_code: KeyCode) -> Self {
        KeyId {
            key_code,
            key_modifiers: KeyModifiers::NONE,
        }
    }
}

impl From<(KeyCode, KeyModifiers)> for KeyId {
    fn from(value: (KeyCode, KeyModifiers)) -> Self {
        KeyId {
            key_code: value.0,
            key_modifiers: value.1,
        }
    }
}
