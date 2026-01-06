use ratatui::termion::event::Key;
use ratatui::termion::input::TermRead;
use ratatui::termion::raw::IntoRawMode;
use ratatui::termion::terminal_size;
use std::env;
use std::io::{Error, Read, Result, Write, stdin, stdout};

enum Mode {
    Normal,
    Edit,
}

#[derive(Default, Clone, Copy)]
struct Location {
    x: usize,
    y: usize,
}

#[derive(Clone)]
struct Buffer {
    lines: Vec<String>,
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
        while line.len() <= loc.x {
            line.push(' ');
        }
        line.insert(loc.x, c);
    }
    pub fn delete_char(&mut self, loc: &Location) -> bool {
        if loc.y == 0 && loc.x == 0 {
            return false;
        }
        let line = &mut self.lines[loc.y];
        if loc.x > 0 {
            line.remove(loc.x - 1);
            true
        } else {
            let prev_line = self.lines.remove(loc.y);
            self.lines[loc.y - 1].push_str(&prev_line);
            true
        }
    }
    pub fn line_at(&self, y: usize) -> &str {
        self.lines.get(y).map(|s| s.as_str()).unwrap_or("")
    }
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
}

#[derive(Default)]
struct View {
    buffer: Buffer,
}
impl View {
    fn render(&self, stdout: &mut std::io::Stdout) -> Result<()> {
        write!(
            stdout,
            "{}{}",
            ratatui::termion::clear::All,
            ratatui::termion::cursor::Goto(1, 1)
        )?;
        for (idx, line) in self.buffer.lines.iter().enumerate() {
            write!(stdout, "{}\r\n", line)?;
        }
        stdout.flush()
    }
}

struct Editor {
    mode: Mode,
    buffer: Buffer,
    view: View,
    location: Location,
    max_cols: usize,
    max_rows: usize,
}

impl Default for Editor {
    fn default() -> Self {
        let (cols, rows) = ratatui::termion::terminal_size().unwrap_or((80, 24));
        Self {
            mode: Mode::Normal,
            buffer: Buffer::default(),
            view: View {
                buffer: Buffer::default(),
            },
            location: Location { x: 0, y: 0 },
            max_cols: cols as usize,
            max_rows: rows as usize,
        }
    }
}

impl Editor {
    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
    fn update_cursor(&self, stdout: &mut std::io::Stdout) -> Result<()> {
        write!(
            stdout,
            "{}",
            ratatui::termion::cursor::Goto(self.location.x as u16 + 1, self.location.y as u16 + 1)
        )?;
        stdout.flush()?;
        Ok(())
    }
    fn move_left(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        if self.location.x > 0 {
            self.location.x -= 1;
        } else if self.location.x > 0 {
            self.location.x -= 1;
            self.location.x = self.buffer.line_at(self.location.y).len();
        }
        self.update_cursor(stdout)?;
        Ok(())
    }
    fn move_right(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        let current_line_len = self.buffer.line_at(self.location.y).len();
        if self.location.x + 1 < current_line_len {
            self.location.x += 1;
        }
        self.update_cursor(stdout)?;
        Ok(())
    }
    fn move_up(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        if self.location.y > 0 {
            self.location.y -= 1;
        }
        if self.location.y >= self.buffer.line_count() {
            self.location.y = self.buffer.line_count().saturating_sub(1);
        }
        self.update_cursor(stdout)?;
        Ok(())
    }
    fn move_down(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        if self.location.y + 1 < self.max_rows {
            self.location.y += 1;
        }
        if self.location.y >= self.buffer.line_count() {
            self.location.y = self.buffer.line_count().saturating_sub(1);
        }
        self.update_cursor(stdout)?;
        Ok(())
    }
    fn handle_cursor(&mut self, key: Key, stdout: &mut std::io::Stdout) -> Result<()> {
        match key {
            Key::Left => self.move_left(stdout)?,
            Key::Right => self.move_right(stdout)?,
            Key::Up => self.move_up(stdout)?,
            Key::Down => self.move_down(stdout)?,
            _ => {}
        }
        Ok(())
    }
    fn run(&mut self) -> Result<()> {
        // let mut size = terminal_size().unwrap();
        let stdin = stdin();
        let mut stdout = stdout().into_raw_mode()?;
        write!(
            stdout,
            "{}{}",
            ratatui::termion::clear::All,
            ratatui::termion::cursor::Goto(1, 1)
        )
        .unwrap();
        stdout.flush().unwrap();
        for k in stdin.keys() {
            let key = k?;
            match self.mode {
                Mode::Normal => match key {
                    // Key::Char('a') => ,
                    Key::Char('i') => self.set_mode(Mode::Edit),
                    // Key::Char('x') => ,
                    // Key::Char('s') => ,
                    // Key::Char('r') => ,
                    // Key::Char('v') => ,
                    Key::Left | Key::Right | Key::Up | Key::Down => {
                        self.handle_cursor(key, &mut stdout)?
                    }
                    Key::Ctrl('q') => break,
                    _ => {}
                },
                Mode::Edit => match key {
                    Key::Char(c) => {
                        self.buffer.insert_char(&self.location, c);
                        self.view.buffer = self.buffer.clone();
                    }
                    Key::Left | Key::Right | Key::Up | Key::Down => {
                        self.handle_cursor(key, &mut stdout)?
                    }
                    Key::Backspace => {
                        if self.buffer.delete_char(&self.location) {
                            self.location.x = self.buffer.line_at(self.location.y).len();
                        }
                        self.view.buffer = self.buffer.clone();
                    }
                    Key::Esc => {
                        self.set_mode(Mode::Normal);
                    }
                    _ => {}
                },
            }
            self.view.render(&mut stdout)?;
            stdout.flush().unwrap();
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut editor = Editor::default();
    editor.run()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_something() {}
}
