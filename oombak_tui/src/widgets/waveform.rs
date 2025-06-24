use bitvec::vec::BitVec;
use oombak_sim::response::{CompactWaveValue, Wave};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::BlockExt,
    style::{Style, Styled},
    widgets::{Block, StatefulWidget, Widget},
};

use crate::{
    components::models::WaveSpec,
    styles::wave_viewer::{CURSOR_STYLE, WAVEFORM_STYLE},
    utils::bitvec_str,
};

use super::ScrollState;

const NUMBER_OF_CELLS_PER_UNIT_TIME: usize = 3;

pub struct Waveform<'a> {
    wave_spec: &'a WaveSpec,
    zoom: u8,
    block: Option<Block<'a>>,
    selected_style: Style,
    is_selected: bool,
}

impl<'a> Waveform<'a> {
    pub fn new(wave_spec: &'a WaveSpec) -> Self {
        Self {
            wave_spec,
            zoom: 0,
            block: None,
            selected_style: Style::default(),
            is_selected: false,
        }
    }

    pub fn zoom(mut self, zoom: u8) -> Self {
        self.zoom = zoom;
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

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl StatefulWidget for Waveform<'_> {
    type State = ScrollState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State)
    where
        Self: Sized,
    {
        state.set_viewport_length(area.width as usize);
        let (compact_values, plot_offset) = self.slice_wave(&self.wave_spec.wave, state);
        let lines = self.plot_values_as_lines(compact_values, plot_offset, state.viewport_length());
        self.render_lines(&lines, area, buf);
        self.add_cursor_highlight(buf, area, state.selected_position(), lines.len() as u16);
    }
}

impl Waveform<'_> {
    fn slice_wave(&self, wave: &Wave, state: &ScrollState) -> (Vec<CompactWaveValue>, usize) {
        if wave.is_empty() {
            return (vec![], 0);
        }

        let mut start_time = state.start_position() / self.unit_width();
        let mut end_time = start_time + (state.viewport_length() / self.unit_width()) + 1;
        end_time = end_time.min(wave.end_time());
        let mut plot_offset = state.start_position() % self.unit_width();

        // To ensure the left end plot does not start with a head when start_time is in the middle
        // of a "stable" value.
        if start_time != 0 {
            start_time -= 1;
            plot_offset += self.unit_width();
        }

        let sliced_wave = wave
            .slice(start_time, end_time)
            .expect("logic error: wave.slice(start_time, end_time) should succeed");

        (sliced_wave, plot_offset)
    }

    fn unit_width(&self) -> usize {
        NUMBER_OF_CELLS_PER_UNIT_TIME * 2usize.pow(self.zoom as u32)
    }

    fn plot_values_as_lines(
        &self,
        compact_values: Vec<CompactWaveValue>,
        plot_offset: usize,
        viewport_length: usize,
    ) -> Vec<String> {
        let height = self.wave_spec.height as usize;
        let mut lines = vec![String::new(); 2 * height + 1];
        for (i, compact_value) in compact_values.iter().enumerate() {
            let is_end_value = i == compact_values.len() - 1;
            let word = self.format(compact_value.value(), compact_value.duration());
            Self::draw_opening(&mut lines, &word, height);
            Self::draw_body(&mut lines, &word, height);
            Self::draw_tail(&mut lines, &word, height, is_end_value);
        }
        lines
            .iter()
            .map(|l| l.chars().skip(plot_offset).take(viewport_length).collect())
            .collect()
    }

    fn render_lines(&self, lines: &[String], area: Rect, buf: &mut Buffer) {
        self.block.render(area, buf);
        let area = self.block.inner_if_some(area);
        let selected_style = if self.is_selected {
            self.selected_style
        } else {
            Style::default()
        };
        for (i, line) in lines.iter().enumerate() {
            let i = i as u16;
            buf.set_string(
                area.x,
                area.y + i,
                line,
                WAVEFORM_STYLE.set_style(selected_style),
            )
        }
    }

    fn add_cursor_highlight(
        &self,
        buf: &mut Buffer,
        area: Rect,
        cursor_position: usize,
        line_count: u16,
    ) {
        buf.set_style(
            Rect::new(area.x + cursor_position as u16, area.y, 1, line_count),
            CURSOR_STYLE,
        );
    }

    fn format(&self, value: &BitVec<u32>, count: usize) -> Vec<char> {
        let option = bitvec_str::Option::from(self.wave_spec);
        let value = bitvec_str::from(value, &option);
        let str_width = NUMBER_OF_CELLS_PER_UNIT_TIME * 2usize.pow(self.zoom as u32) * count + 1;
        let res = if str_width - 2 >= value.len() {
            format!("{:^1$}", value, str_width)
        } else {
            let snip_size = usize::saturating_sub(str_width, 3);
            let snip = &value[0..snip_size];
            format!(" {snip}~ ")
        };
        res.chars().take(str_width).collect()
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
}
