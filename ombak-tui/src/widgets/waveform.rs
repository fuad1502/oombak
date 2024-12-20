use bitvec::vec::BitVec;
use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

use crate::utils::bitvec_str;

pub struct Waveform {
    values: Vec<BitVec>,
    height: u16,
    width: u8,
    option: bitvec_str::Option,
}

impl Waveform {
    pub fn new(values: Vec<BitVec>, height: u16, width: u8, option: bitvec_str::Option) -> Self {
        Self {
            values,
            height,
            width,
            option,
        }
    }

    fn format(&self, value: &BitVec) -> Vec<char> {
        let value = bitvec_str::from(value, &self.option);
        let width = self.width as usize;
        let res = if width >= value.len() {
            format!("{:^1$}", value, width)
        } else {
            let snip_size = usize::saturating_sub(width, 3);
            let snip = &value[0..snip_size];
            format!("{snip}...")
        };
        res.chars().take(width).collect()
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
}

impl Widget for Waveform {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let style = Style::default();
        let height = self.height as usize;
        let mut lines = vec![String::new(); 2 * height + 1];
        for (c, value) in self.values.iter().enumerate() {
            let is_end_value = c == self.values.len() - 1;
            let word = self.format(value);
            Self::draw_opening(&mut lines, height);
            Self::draw_body(&mut lines, &word, height);
            Self::draw_tail(&mut lines, height, is_end_value);
        }
        for (i, line) in lines.iter().enumerate() {
            let i = i as u16;
            buf.set_string(area.x, area.y + i, line, style);
        }
    }
}
