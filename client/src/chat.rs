use crossterm::style::Color;

pub struct MessagePiece {
    contents: String,
    fg_color: Color,
    bg_color: Color,
}

impl MessagePiece {
    fn new(contents: String, fg_color: Color, bg_color: Color) -> Self {
        Self {
            contents,
            fg_color,
            bg_color,
        }
    }

    fn message(contents: String) -> Self {
        Self::new(contents, Color::White, Color::Black)
    }

    fn username(contents: String) -> Self {
        Self::new(contents, Color::Green, Color::Black)
    }

    fn timestamp(contents: String) -> Self {
        Self::new(contents, Color::Grey, Color::Black)
    }
}

pub struct ChatMessage {
    timestamp: MessagePiece,
    username: MessagePiece,
    msg: MessagePiece,
    bg_color: Color,
    fg_color: Color,
}

impl ChatMessage {
    fn new(msg: String, timestamp: String, username: String, fg_color: Color, bg_color: Color) -> Self {
        Self {
            timestamp: MessagePiece::timestamp(timestamp),
            username: MessagePiece::username(username),
            msg: MessagePiece::message(msg),
            fg_color,
            bg_color,
        }
    }

    fn default(msg: String, timestamp: String, username: String) -> Self {
        Self::new(msg, timestamp, username, Color::White, Color::Black)
    }

    fn system(msg: String, timestamp: String, username: String) -> Self {
        Self::new(msg, timestamp, username, Color::Red, Color::Black)
    }
}

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