extern crate shared;

pub struct ClientInput {
    // TODO: make private when the flux move is over
    pub inner: Vec<char>,
    cursor: usize,
}

impl ClientInput {
    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
            cursor: 0,
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.inner.remove(self.cursor);
        }
    }

    pub fn backspace_forward(&mut self) {
        if self.inner.len() > 0 && self.cursor < self.inner.len() {
            self.inner.remove(self.cursor);
        }
    }

    pub fn insert(&mut self, elem: char) {
        self.inner.insert(self.cursor, elem);
        self.cursor += 1;
    }

    pub fn insert_uppercase(&mut self, ch: std::char::ToUppercase) {
        for (idx, c) in ch.enumerate() {
            self.inner.insert(self.cursor + idx, c);
        }
        self.cursor += 1;
    }

    pub fn left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn right(&mut self) {
        if self.cursor < self.inner.len() {
            self.cursor += 1;
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.cursor = 0;
    }

    pub fn position(&self) -> usize {
        self.cursor
    }

    pub fn get_ref(&self) -> &[char] {
        &self.inner
    }
}
