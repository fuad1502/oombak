use std::collections::{btree_map, hash_map, BTreeMap, HashMap};

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

impl CommandKeysHelpBar<'_> {
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
        for (k, v) in lower_prio {
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
        for (k, v) in key_maps {
            if let Some(key_ids) = reversed_key_maps.get_mut(v) {
                key_ids.push(k.clone());
            } else {
                reversed_key_maps.insert(v.to_string(), vec![k.clone()]);
            }
        }
        ReversedKeyMaps(reversed_key_maps)
    }
}

impl<'a> From<(&'a String, &'a Vec<KeyId>)> for KeyMap {
    fn from(value: (&'a String, &'a Vec<KeyId>)) -> Self {
        KeyMap {
            key_ids: value.1.to_vec(),
            description: value.0.to_string(),
        }
    }
}

impl<'a> From<&'a KeyMap> for Line<'a> {
    fn from(value: &'a KeyMap) -> Self {
        Line::default().spans(Vec::from(value))
    }
}

impl<'a> From<&'a KeyMap> for Vec<Span<'a>> {
    fn from(value: &'a KeyMap) -> Self {
        let key_ids = value.key_ids_to_string();
        vec![
            Span::from("["),
            Span::from(key_ids).style(KEY_ID_STYLE),
            Span::from(": "),
            Span::from(value.description.clone()).style(DESCRIPTION_STYLE),
            Span::from("] "),
        ]
    }
}

impl KeyMap {
    fn key_ids_to_string(&self) -> String {
        if self.key_ids.len() > 1 {
            self.key_ids
                .iter()
                .map(KeyId::to_string)
                .collect::<Vec<String>>()
                .join(", ")
        } else if !self.key_ids.is_empty() {
            self.key_ids[0].to_string()
        } else {
            "".to_string()
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

impl From<&ReversedKeyMaps> for Vec<KeyMap> {
    fn from(value: &ReversedKeyMaps) -> Self {
        value.into_iter().map(KeyMap::from).collect()
    }
}

impl<'a> IntoIterator for &'a KeyMaps {
    type Item = (&'a KeyId, &'a String);

    type IntoIter = hash_map::Iter<'a, KeyId, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a ReversedKeyMaps {
    type Item = (&'a String, &'a Vec<KeyId>);

    type IntoIter = btree_map::Iter<'a, String, Vec<KeyId>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
