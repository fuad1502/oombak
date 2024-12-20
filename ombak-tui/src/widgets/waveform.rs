use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

pub struct Waveform {
    values: Vec<String>,
    height: u16,
    width: u8,
}

impl Waveform {
    pub fn new(values: Vec<String>, height: u16, width: u8) -> Self {
        Self {
            values,
            height,
            width,
        }
    }

    fn format(&self, value: &str) -> Vec<char> {
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
}

impl Widget for Waveform {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let style = Style::default();
        let height = self.height as usize;
        let mut lines = vec![String::new(); height * 2 + 1];
        for value in &self.values {
            let word = self.format(value);
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
            for i in 0..height + 1 {
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
        for i in 0..(self.height * 2 + 1) {
            buf.set_string(area.x, area.y + i, lines[i as usize].clone(), style);
        }
    }
}
