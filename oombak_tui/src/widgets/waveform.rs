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
    components::models::{PlotType, WaveSpec},
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
        let lines = match (&self.wave_spec.plot_type, self.wave_spec.wave.width) {
            (PlotType::Analog, _) | (PlotType::Digital, 1) => {
                self.analog_plot(compact_values, plot_offset, state.viewport_length())
            }
            _ => self.digital_plot(compact_values, plot_offset, state.viewport_length()),
        };
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

    fn digital_plot(
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
        Self::trim_plot_start(&lines, plot_offset, viewport_length)
    }

    fn analog_plot(
        &self,
        compact_values: Vec<CompactWaveValue>,
        plot_offset: usize,
        viewport_length: usize,
    ) -> Vec<String> {
        let height = self.wave_spec.height as usize;
        let num_of_levels = 2 * height + 1;
        let mut lines = vec![String::new(); num_of_levels];
        let level_mapper = AnalogLevelMapper::new(&compact_values, num_of_levels);
        for (compact_value, next_compact_value) in
            compact_values.iter().zip(compact_values.iter().skip(1))
        {
            let level = level_mapper.map(compact_value.value());
            let next_level = level_mapper.map(next_compact_value.value());
            let duration = compact_value.duration() * self.unit_width();
            Self::draw_level(&mut lines, level, duration, next_level);
        }
        if let Some(compact_value) = compact_values.last() {
            let level = level_mapper.map(compact_value.value());
            let duration = compact_value.duration() * self.unit_width();
            Self::draw_level(&mut lines, level, duration, level);
        }
        Self::trim_plot_start(&lines, plot_offset, viewport_length)
    }

    fn draw_level(lines: &mut Vec<String>, level: usize, duration: usize, next_level: usize) {
        let target_row = lines.len() - level - 1;
        for _ in 0..(duration - 1) {
            for (row, line) in lines.iter_mut().enumerate() {
                if row == target_row {
                    *line += "─";
                } else {
                    *line += " ";
                }
            }
        }
        Self::draw_level_transition(lines, level, next_level);
    }

    fn draw_level_transition(lines: &mut Vec<String>, level: usize, next_level: usize) {
        let is_increasing = next_level > level;
        let is_decreasing = next_level < level;
        let start_row = lines.len() - level - 1;
        let end_row = lines.len() - next_level - 1;
        for (row, line) in lines.iter_mut().enumerate() {
            if row == start_row && is_increasing {
                *line += "┘";
            } else if row == start_row && is_decreasing {
                *line += "┐";
            } else if row == end_row && is_increasing {
                *line += "┌";
            } else if row == end_row && is_decreasing {
                *line += "└";
            } else if is_increasing && (row > end_row && row < start_row) {
                *line += "│";
            } else if is_decreasing && (row < end_row && row > start_row) {
                *line += "│";
            } else if row == start_row && !is_increasing && !is_decreasing {
                *line += "─";
            } else {
                *line += " ";
            }
        }
    }

    fn trim_plot_start(
        lines: &[String],
        plot_offset: usize,
        viewport_length: usize,
    ) -> Vec<String> {
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

struct AnalogLevelMapper {
    limits: Vec<f64>,
    min_value: f64,
}

impl AnalogLevelMapper {
    fn new(compact_values: &Vec<CompactWaveValue>, num_of_levels: usize) -> Self {
        let (limits, min_value) = Self::calculate_limits(compact_values, num_of_levels);
        Self { limits, min_value }
    }

    fn calculate_limits(
        compact_values: &Vec<CompactWaveValue>,
        num_of_levels: usize,
    ) -> (Vec<f64>, f64) {
        if compact_values.is_empty() {
            return (vec![], 0f64);
        }

        let values = compact_values.iter().map(|v| Self::u32_from(v.value()));
        let max_value = values.clone().max().unwrap();
        let min_value = values.min().unwrap();
        let limits = (1..num_of_levels)
            .map(|i| i as u32)
            .map(|i| (i * (max_value - min_value)) as f64 / num_of_levels as f64)
            .collect();

        (limits, min_value as f64)
    }

    fn map(&self, value: &BitVec<u32>) -> usize {
        self.limits
            .partition_point(|l| *l <= Self::u32_from(value) as f64 - self.min_value)
    }

    fn u32_from(value: &BitVec<u32>) -> u32 {
        match value.last_one() {
            Some(i) if i > 31 => unimplemented!(),
            Some(_) => value.clone().into_vec()[0],
            None => 0u32,
        }
    }
}
