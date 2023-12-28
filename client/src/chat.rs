pub struct ChatLog {
    lines: Vec<String>,
    height: usize,
    width: usize,
}

impl ChatLog {
    pub fn new(height: usize, width: usize) -> Self {
        Self {
            lines: Vec::new(),
            height,
            width,
        }
    }

    pub fn put_line(&mut self, line: String) {
        // TODO: make it limited to height
        self.lines.push(line);
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn get(&self) -> &[String] {
        &self.lines
    }
}