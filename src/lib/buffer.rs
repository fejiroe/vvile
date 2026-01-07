use std::fs;
use std::io::{Error, Result};
use std::path::Path;

#[derive(Default, Clone, Copy)]
pub struct Location {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone)]
pub struct Buffer {
    pub lines: Vec<String>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
        }
    }
}

impl Buffer {
    pub fn insert_char(&mut self, loc: &Location, c: char) {
        let line = &mut self.lines[loc.y];
        line.insert(loc.x, c);
    }
    pub fn delete_char(&mut self, loc: &Location) -> bool {
        if loc.y == 0 && loc.x == 0 {
            return false;
        }
        if loc.x > 0 {
            let line = &mut self.lines[loc.y];
            line.remove(loc.x - 1);
        } else {
            let current_line = self.lines.remove(loc.y);
            self.lines[loc.y - 1].push_str(&current_line);
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
            return String::new();
        } else {
            return self.lines.join("\n");
        }
    }
    pub fn read_file(&mut self, path: &Path) -> Result<()> {
        let contents = fs::read_to_string(path)?;
        let input = contents
            .lines()
            .map(|l| l.to_owned())
            .collect::<Vec<String>>();
        self.lines = input;
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        Ok(())
    }
    pub fn write_file(&self, path: &Path) -> Result<()> {
        let out = self.buffer_to_string();
        fs::write(path, out)?;
        Ok(())
    }
}
