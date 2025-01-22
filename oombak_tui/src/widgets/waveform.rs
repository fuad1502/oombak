use bitvec::vec::BitVec;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::BlockExt,
    style::{Style, Stylize},
    widgets::{Block, Widget},
};

use crate::{components::models::WaveSpec, utils::bitvec_str};

pub struct Waveform<'a> {
    values: &'a Vec<BitVec<u32>>,
    height: u16,
    width: u8,
    highlighted_idx: u16,
    option: bitvec_str::Option,
    block: Option<Block<'a>>,
    selected_style: Style,
    is_selected: bool,
}

impl<'a> From<&'a WaveSpec> for Waveform<'a> {
    fn from(wave_spec: &'a WaveSpec) -> Self {
        let option = bitvec_str::Option::from(wave_spec);
        Self {
            values: &wave_spec.wave.values,
            height: wave_spec.height,
            width: 0,
            highlighted_idx: 0,
            option,
            block: None,
            selected_style: Style::default(),
            is_selected: false,
        }
    }
}

impl<'a> Waveform<'a> {
    pub fn new(
        values: &'a Vec<BitVec<u32>>,
        height: u16,
        width: u8,
        option: bitvec_str::Option,
    ) -> Self {
        Self {
            values,
            height,
            width,
            highlighted_idx: 0,
            option,
            block: None,
            selected_style: Style::default(),
            is_selected: false,
        }
    }

    pub fn width(mut self, width: u8) -> Self {
        self.width = width;
        self
    }

    pub fn highlight(mut self, idx: u16) -> Self {
        self.highlighted_idx = idx;
        self
    }

    pub fn selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    pub fn selected(mut self, is_selected: bool) -> Self {
        self.is_selected = is_selected;
        self
    }

    fn format(&self, value: &BitVec<u32>, count: usize) -> Vec<char> {
        let value = bitvec_str::from(value, &self.option);
        let width = self.width as usize;
        let height = self.height as usize;
        let body_length = width * count;
        let head_tail_lengths = 2 * (height + 1) * count;
        let overlaps = count - 1;
        let str_width = body_length + head_tail_lengths - overlaps;
        let res = if str_width - 2 >= value.len() {
            format!("{:^1$}", value, str_width)
        } else {
            let snip_size = usize::saturating_sub(str_width, 3);
            let snip = &value[0..snip_size];
            format!(" {snip}~ ")
        };
        res.chars().take(str_width).collect()
    }

    fn compact_vec(values: &[BitVec<u32>]) -> Vec<(BitVec<u32>, usize)> {
        let (_, values, counts) = values.iter().fold(
            (None, vec![], vec![]),
            |(prev, mut values, mut counts), x| {
                if Some(x) == prev {
                    *counts.last_mut().unwrap() += 1;
                    (prev, values, counts)
                } else {
                    values.push(x.clone());
                    counts.push(1);
                    (Some(x), values, counts)
                }
            },
        );
        values.into_iter().zip(counts).collect()
    }

    fn draw_opening(lines: &mut [String], word: &[char], height: usize) {
        let head_length = height + 1;
        for (i, c) in word.iter().take(head_length).enumerate() {
            for (j, line) in lines.iter_mut().enumerate() {
                if i == 0 && j == height {
                    *line += "\u{2573}";
                } else if i > 0 && j == height {
                    *line += &format!("{}", c);
                } else if i > 0 && j == height - i {
                    *line += "\u{2571}";
                } else if i > 0 && j == height + i {
                    *line += "\u{2572}";
                } else {
                    *line += " ";
                }
            }
        }
    }

    fn draw_tail(lines: &mut [String], word: &[char], height: usize, is_end_value: bool) {
        let head_length = height + 1;
        let tail_length = if is_end_value { height + 1 } else { height };
        for (i, c) in word
            .iter()
            .skip(word.len() - head_length)
            .take(tail_length)
            .enumerate()
        {
            for (j, line) in lines.iter_mut().enumerate() {
                if i == height && j == height {
                    *line += "\u{2573}";
                } else if i < height && j == height {
                    *line += &format!("{}", c);
                } else if i < height && j == i {
                    *line += "\u{2572}";
                } else if i < height && j == 2 * height - i {
                    *line += "\u{2571}";
                } else {
                    *line += " ";
                }
            }
        }
    }

    fn draw_body(lines: &mut [String], word: &[char], height: usize) {
        let head_length = height + 1;
        let body_length = word.len() - 2 * head_length;
        for c in word.iter().skip(head_length).take(body_length) {
            for (j, line) in lines.iter_mut().enumerate() {
                if j == height {
                    *line += &format!("{}", c);
                } else if j == 0 {
                    *line += "\u{2594}";
                } else if j == height * 2 {
                    *line += "\u{2581}";
                } else {
                    *line += " ";
                }
            }
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl Widget for Waveform<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let style = Style::default();
        let height = self.height as usize;
        let mut lines = vec![String::new(); 2 * height + 1];
        let value_count_pair = Self::compact_vec(self.values);
        for (c, (value, count)) in value_count_pair.iter().enumerate() {
            let is_end_value = c == value_count_pair.len() - 1;
            let word = self.format(value, *count);
            Self::draw_opening(&mut lines, &word, height);
            Self::draw_body(&mut lines, &word, height);
            Self::draw_tail(&mut lines, &word, height, is_end_value);
        }
        self.block.render(area, buf);
        let area = self.block.inner_if_some(area);
        for (i, line) in lines.iter().enumerate() {
            let i = i as u16;
            buf.set_string(area.x, area.y + i, line, style);
        }
        if self.is_selected {
            buf.set_style(area, self.selected_style);
        }
        buf.set_style(
            Rect::new(area.x + self.highlighted_idx, area.y, 1, lines.len() as u16),
            Style::default().on_red(),
        );
    }
}
