use bitvec::vec::BitVec;
use oombak_sim::sim::Wave;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::BlockExt,
    style::{Style, Stylize},
    widgets::{Block, StatefulWidget, Widget},
};

use crate::{components::models::WaveSpec, utils::bitvec_str};

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
    type State = WaveformScrollState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State)
    where
        Self: Sized,
    {
        state.set_viewport_length(area.width as usize);
        let (value_count_pairs, start_skip) = self.trim_wave_values(&self.wave_spec.wave, state);
        let lines = self.plot_values_as_lines(value_count_pairs, start_skip, state.viewport_length);
        self.render_lines(&lines, area, buf);
        self.add_cursor_highlight(buf, area, state.selected_position, lines.len() as u16);
    }
}

impl Waveform<'_> {
    fn trim_wave_values(
        &self,
        wave: &Wave,
        state: &WaveformScrollState,
    ) -> (Vec<(BitVec<u32>, usize)>, usize) {
        let unit_size = NUMBER_OF_CELLS_PER_UNIT_TIME * 2usize.pow(self.zoom as u32);
        let start_time = state.start_position / unit_size;
        let mut start_cut = state.start_position % unit_size;
        let end_time =
            (state.start_position + usize::saturating_sub(state.viewport_length, 1)) / unit_size;
        let mut result = vec![];

        if let Some((start_idx, start_offset)) = wave.value_idx_at(start_time) {
            let (end_idx, end_offset) = wave
                .value_idx_at(end_time)
                .unwrap_or((wave.values.len() - 1, wave.values.last().unwrap().2 - 1));

            let start_value = wave.values[start_idx].0.clone();
            let mut start_count = usize::min(
                wave.values[start_idx].2 - start_offset,
                end_time - start_time + 1,
            );
            if start_offset != 0 {
                start_count += 1;
                start_cut += unit_size;
            }
            if end_idx == start_idx && end_offset != wave.values[end_idx].2 - 1 {
                start_count += 1;
            }
            result.push((start_value, start_count));

            for i in start_idx + 1..=end_idx {
                let value = wave.values[i].0.clone();
                let count = if i == end_idx && end_offset != wave.values[i].2 - 1 {
                    end_offset + 2
                } else if i == end_idx {
                    end_offset + 1
                } else {
                    wave.values[i].2
                };
                result.push((value, count));
            }
        }

        (result, start_cut)
    }

    fn plot_values_as_lines(
        &self,
        value_count_pairs: Vec<(BitVec<u32>, usize)>,
        skip_start: usize,
        viewport_length: usize,
    ) -> Vec<String> {
        let height = self.wave_spec.height as usize;
        let mut lines = vec![String::new(); 2 * height + 1];
        for (c, (value, count)) in value_count_pairs.iter().enumerate() {
            let is_end_value = c == value_count_pairs.len() - 1;
            let word = self.format(value, *count);
            Self::draw_opening(&mut lines, &word, height);
            Self::draw_body(&mut lines, &word, height);
            Self::draw_tail(&mut lines, &word, height, is_end_value);
        }
        lines
            .iter()
            .map(|l| l.chars().skip(skip_start).take(viewport_length).collect())
            .collect()
    }

    fn render_lines(&self, lines: &[String], area: Rect, buf: &mut Buffer) {
        self.block.render(area, buf);
        let area = self.block.inner_if_some(area);
        let style = if self.is_selected {
            self.selected_style
        } else {
            Style::default()
        };
        for (i, line) in lines.iter().enumerate() {
            let i = i as u16;
            buf.set_string(area.x, area.y + i, line, style);
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
            Style::default().on_red(),
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

#[derive(Default, Clone)]
pub struct WaveformScrollState {
    start_position: usize,
    selected_position: usize,
    content_length: usize,
    viewport_length: usize,
}

impl WaveformScrollState {
    pub fn new(content_length: usize) -> Self {
        Self {
            start_position: 0,
            selected_position: 0,
            content_length,
            viewport_length: 0,
        }
    }

    pub fn set_content_length(&mut self, content_length: usize) {
        self.content_length = content_length;
    }

    pub fn set_viewport_length(&mut self, viewport_length: usize) {
        let viewport_length = usize::min(viewport_length, self.content_length);
        if self.selected_position >= viewport_length {
            self.selected_position = usize::saturating_sub(self.viewport_length, 1);
        }
        self.viewport_length = viewport_length;
    }

    pub fn next(&mut self) {
        if !self.is_at_end() && self.is_at_viewport_end() {
            self.start_position += 1;
        } else if !self.is_at_viewport_end() {
            self.selected_position += 1;
        }
    }

    pub fn prev(&mut self) {
        if !self.is_at_beginning() && self.is_at_viewport_start() {
            self.start_position -= 1;
        } else if !self.is_at_viewport_start() {
            self.selected_position -= 1;
        }
    }

    fn is_at_viewport_end(&self) -> bool {
        self.viewport_length == 0 || self.selected_position == self.viewport_length - 1
    }

    fn is_at_viewport_start(&self) -> bool {
        self.selected_position == 0
    }

    fn is_at_end(&self) -> bool {
        self.content_length == 0
            || (self.start_position == self.content_length - self.viewport_length
                && self.selected_position == self.viewport_length - 1)
    }

    fn is_at_beginning(&self) -> bool {
        self.start_position == 0 && self.selected_position == 0
    }
}
