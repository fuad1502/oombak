#[derive(Default, Clone, Copy)]
pub struct ScrollState {
    content_length: usize,
    start_position: usize,
    selected_position: usize,
    viewport_length: usize,
}

impl ScrollState {
    pub fn start_position(&self) -> usize {
        self.start_position
    }

    pub fn selected_position(&self) -> usize {
        self.selected_position
    }

    pub fn viewport_length(&self) -> usize {
        self.viewport_length
    }

    pub fn set_content_length(&mut self, content_length: usize) {
        self.content_length = content_length;
        if self.start_position + self.selected_position >= content_length {
            self.last();
        }
    }

    pub fn set_viewport_length(&mut self, viewport_length: usize) {
        self.viewport_length = viewport_length;
        let excess_blanks = (self.start_position + self.viewport_length)
            .saturating_sub(self.content_length)
            .min(self.start_position);
        if excess_blanks > 0 {
            self.start_position -= excess_blanks;
            self.selected_position += excess_blanks;
        }
        if self.selected_position >= viewport_length {
            self.selected_position = viewport_length - 1;
        }
    }

    pub fn next(&mut self) {
        if !self.is_at_end() && self.is_at_viewport_end() {
            self.start_position += 1;
        } else if !self.is_at_end() && !self.is_at_viewport_end() {
            self.selected_position += 1;
        }
    }

    pub fn prev(&mut self) {
        if !self.is_at_beginning() && self.is_at_viewport_start() {
            self.start_position -= 1;
        } else if !self.is_at_beginning() && !self.is_at_viewport_start() {
            self.selected_position -= 1;
        }
    }

    pub fn last(&mut self) {
        let effective_viewport_length = usize::min(self.viewport_length, self.content_length);
        self.start_position = self.content_length - effective_viewport_length;
        self.selected_position = effective_viewport_length.saturating_sub(1);
    }

    fn is_at_viewport_end(&self) -> bool {
        self.selected_position == self.viewport_length.saturating_sub(1)
    }

    fn is_at_viewport_start(&self) -> bool {
        self.selected_position == 0
    }

    fn is_at_end(&self) -> bool {
        self.start_position + self.selected_position == self.content_length.saturating_sub(1)
    }

    fn is_at_beginning(&self) -> bool {
        self.start_position == 0 && self.selected_position == 0
    }
}
