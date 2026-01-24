use std::fs;
use std::io::Result;
use std::path::Path;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default, Clone, Copy)]
pub struct Location {
    pub x: usize,
    pub y: usize,
}

#[derive(Default, Debug, Clone)]
pub struct Line {
    pub raw: String,
    pub graphemes: Vec<usize>,
}

impl Line {
    fn new() -> Self {
        Self {
            raw: String::new(),
            graphemes: vec![0],
        }
    }
    fn rebuild(&mut self) {
        let mut offsets = Vec::new();
        for (i, _) in self.raw.grapheme_indices(true) {
            offsets.push(i);
        }
        offsets.push(self.raw.len());
        self.graphemes = offsets;
    }
    pub fn insert(&mut self, i: usize, c: char) {
        let byte_offset = self.graphemes[i];
        self.raw.insert(byte_offset, c);
        self.rebuild();
    }
    pub fn remove(&mut self, i: usize) {
        let start = self.graphemes[i];
        let end = self.graphemes[i + 1];
        self.raw.replace_range(start..end, "");
        self.rebuild();
    }
    pub fn push_str(&mut self, s: &str) {
        self.raw.push_str(s);
        self.rebuild();
    }
    pub fn as_str(&self) -> &str {
        &self.raw
    }
    pub fn from_string(s: String) -> Self {
        let mut l = Self::new();
        l.raw = s;
        l.rebuild();
        l
    }
    pub fn grapheme_at(&self, i: usize) -> Option<&str> {
        let start = *self.graphemes.get(i)?;
        let end = *self.graphemes.get(i + 1)?;
        Some(&self.raw[start..end])
    }
    pub fn grapheme_len(&self) -> usize {
        self.graphemes.len().saturating_sub(1)
    }
}

#[derive(Clone, Debug)]
pub struct Buffer {
    pub lines: Vec<Line>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            lines: vec![Line::new()],
        }
    }
}

impl Buffer {
    pub fn insert_char(&mut self, loc: &Location, c: char) {
        if self.lines.is_empty() {
            self.lines.push(Line::new());
        }
        while self.lines.len() <= loc.y {
            self.lines.push(Line::new());
        }
        let line = &mut self.lines[loc.y];
        line.insert(loc.x, c);
    }
    pub fn delete_char(&mut self, loc: &Location) -> bool {
        if loc.y == 0 && loc.x == 0 {
            return false;
        }
        if self.lines.is_empty() {
            self.lines.push(Line::new());
        }
        if loc.x > 0 {
            let line = &mut self.lines[loc.y];
            line.remove(loc.x - 1);
        } else {
            let current_line = self.lines.remove(loc.y);
            self.lines[loc.y - 1].push_str(&current_line.raw);
        }
        true
    }
    pub fn line_at(&self, y: usize) -> &str {
        self.lines.get(y).map(|s| s.as_str()).unwrap_or("")
    }
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
    pub fn buffer_to_string(&self) -> String {
        if self.is_empty() {
            String::new()
        } else {
            self.lines
                .iter()
                .map(|l| l.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
    pub fn read_file(&mut self, path: &Path) -> Result<()> {
        let contents = fs::read_to_string(path)?;
        let input = contents
            .lines()
            .map(|l| Line::from_string(l.to_owned()))
            .collect::<Vec<Line>>();
        self.lines = input;
        if contents.ends_with('\n') {
            self.lines.push(Line::new());
        }
        if self.lines.is_empty() {
            self.lines.push(Line::new());
        }
        Ok(())
    }
}
