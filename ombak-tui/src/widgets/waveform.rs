use bitvec::vec::BitVec;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::BlockExt,
    style::Style,
    widgets::{Block, Widget},
};

use crate::utils::bitvec_str;

pub struct Waveform<'a> {
    values: Vec<BitVec>,
    height: u16,
    width: u8,
    option: bitvec_str::Option,
    block: Option<Block<'a>>,
}

impl<'a> Waveform<'a> {
    pub fn new(values: Vec<BitVec>, height: u16, width: u8, option: bitvec_str::Option) -> Self {
        Self {
            values,
            height,
            width,
            option,
            block: None,
        }
    }

    fn format(&self, value: &BitVec, count: usize) -> Vec<char> {
        let value = bitvec_str::from(value, &self.option);
        let width = self.width as usize;
        let height = self.height as usize;
        let str_width = (width * count) + (2 * height + 1) * (count - 1);
        let res = if str_width >= value.len() {
            format!("{:^1$}", value, str_width)
        } else {
            let snip_size = usize::saturating_sub(str_width, 3);
            let snip = &value[0..snip_size];
            format!("{snip}...")
        };
        res.chars().take(str_width).collect()
    }

    fn compact_vec(values: &[BitVec]) -> Vec<(BitVec, usize)> {
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

    fn draw_opening(lines: &mut [String], height: usize) {
        for i in 0..height + 1 {
            for (j, line) in lines.iter_mut().enumerate() {
                if i == 0 && j == height {
                    *line += "\u{2573}";
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

    fn draw_tail(lines: &mut [String], height: usize, is_end_value: bool) {
        let tail_length = if is_end_value { height + 1 } else { height };
        for i in 0..tail_length {
            for (j, line) in lines.iter_mut().enumerate() {
                if i == height && j == height {
                    *line += "\u{2573}";
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

    fn draw_body(lines: &mut [String], word: &Vec<char>, height: usize) {
        for c in word {
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

impl<'a> Widget for Waveform<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let style = Style::default();
        let height = self.height as usize;
        let mut lines = vec![String::new(); 2 * height + 1];
        let value_count_pair = Self::compact_vec(&self.values);
        for (c, (value, count)) in value_count_pair.iter().enumerate() {
            let is_end_value = c == value_count_pair.len() - 1;
            let word = self.format(value, *count);
            Self::draw_opening(&mut lines, height);
            Self::draw_body(&mut lines, &word, height);
            Self::draw_tail(&mut lines, height, is_end_value);
        }
        self.block.render(area, buf);
        let area = self.block.inner_if_some(area);
        for (i, line) in lines.iter().enumerate() {
            let i = i as u16;
            buf.set_string(area.x, area.y + i, line, style);
        }
    }
}
