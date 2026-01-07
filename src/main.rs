use ratatui::symbols::line;
use ratatui::termion::event::Key;
use ratatui::termion::input::TermRead;
use ratatui::termion::raw::IntoRawMode;
use ratatui::termion::terminal_size;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::io::{Error, Read, Result, Write, stdin, stdout};
use std::path::{Path, PathBuf};

enum Mode {
    Normal,
    Edit,
    Command,
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
    fn insert_char(&mut self, loc: &Location, c: char) {
        let line = &mut self.lines[loc.y];
        line.insert(loc.x, c);
    }
    fn delete_char(&mut self, loc: &Location) -> bool {
        if loc.y == 0 && loc.x == 0 {
            return false;
        }
        if loc.x > 0 {
            let line = &mut self.lines[loc.y];
            line.remove(loc.x - 1);
        } else {
            let current_line = self.lines.remove(loc.y);
            self.lines[loc.y - 1].push_str(&current_line);
            let new_y = loc.y - 1;
            let new_x = std::cmp::min(loc.x, self.lines[new_y].len());
        }
        true
    }
    fn line_at(&self, y: usize) -> &str {
        self.lines.get(y).map(|s| s.as_str()).unwrap_or("")
    }
    fn line_count(&self) -> usize {
        self.lines.len()
    }
    fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
    fn buffer_to_string(&self) -> String {
        if self.is_empty() {
            return String::new();
        } else {
            return self.lines.join("\\n");
        }
    }
    fn read_file(&mut self, path: &Path) -> Result<()> {
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
    fn write_file(&self, path: &Path) -> Result<()> {
        let out = self.buffer_to_string();
        fs::write(path, out)?;
        Ok(())
    }
}

#[derive(Default)]
struct View {
    buffer: Buffer,
    needs_update: bool,
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
    current_file: PathBuf,
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
            current_file: PathBuf::new(),
            mode: Mode::Normal,
            buffer: Buffer::default(),
            view: View {
                buffer: Buffer::default(),
                needs_update: false,
            },
            location: Location { x: 0, y: 0 },
            max_cols: cols as usize,
            max_rows: rows as usize,
        }
    }
}

impl Editor {
    fn open_file(&mut self, at: &Path) -> Result<()> {
        self.current_file = at.to_path_buf();
        match self.buffer.read_file(at) {
            Ok(()) => {}
            Err(e) if e.kind() == ErrorKind::NotFound => {
                self.buffer = Buffer::default();
            }
            Err(e) => return Err(e),
        }
        self.view.buffer = self.buffer.clone();
        Ok(())
    }
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
        let line_len = self.buffer.line_at(self.location.y).len();
        if self.location.x > 0 {
            self.location.x -= 1;
        } else if self.location.y > 0 {
            self.location.y -= 1;
            self.location.x = line_len;
        }
        self.update_cursor(stdout)?;
        Ok(())
    }
    fn move_right(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        let line_len = self.buffer.line_at(self.location.y).len();
        if self.location.x + 1 < line_len {
            self.location.x += 1;
        } else if self.location.y + 1 < self.buffer.line_count() {
            self.location.y += 1;
            self.location.x = 0;
        }
        self.update_cursor(stdout)?;
        Ok(())
    }
    fn move_up(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        if self.location.y > 0 {
            self.location.y -= 1;
        }
        let line_len = self.buffer.line_at(self.location.y).len();
        if self.location.x > line_len {
            self.location.x = line_len;
        }
        self.update_cursor(stdout)?;
        Ok(())
    }
    fn move_down(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        let last_line = self.buffer.line_count().saturating_sub(1);
        if self.location.y < last_line {
            self.location.y += 1;
        }
        let line_len = self.buffer.line_at(self.location.y).len();
        if self.location.x >= line_len {
            self.location.x = line_len;
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
                    // Key::Char('u') => ,
                    // Key::Char('v') => ,
                    // Key::Char('/') => ,
                    // Key::Char('?') => ,
                    Key::Left | Key::Right | Key::Up | Key::Down => {
                        self.handle_cursor(key, &mut stdout)?
                    }
                    Key::Ctrl('s') => self.buffer.write_file(&self.current_file)?,
                    Key::Ctrl('q') => break,
                    _ => {}
                },
                Mode::Edit => match key {
                    Key::Char('\n') => {
                        let cur_line = self.buffer.line_at(self.location.y).to_owned();
                        let (left, right) = cur_line.split_at(self.location.x);
                        self.buffer.lines[self.location.y] = left.to_string();
                        self.buffer
                            .lines
                            .insert(self.location.y + 1, right.to_string());
                        self.location.y += 1;
                        self.location.x = 0;
                        self.view.buffer = self.buffer.clone();
                        self.update_cursor(&mut stdout)?;
                    }
                    Key::Char('\t') => {
                        let tab_width = 4;
                        for _ in 0..tab_width {
                            self.buffer.insert_char(&self.location, ' ');
                        }
                        let target_col =
                            ((self.location.x + tab_width - 1) / tab_width) * tab_width;
                        self.location.x = target_col;
                        self.update_cursor(&mut stdout)?;
                        self.view.buffer = self.buffer.clone();
                    }
                    Key::Char(c) => {
                        self.buffer.insert_char(&self.location, c);
                        self.location.x += 1;
                        self.update_cursor(&mut stdout)?;
                        self.view.buffer = self.buffer.clone();
                    }
                    Key::Backspace => {
                        if self.buffer.delete_char(&self.location) {
                            if self.location.x == 0 && self.location.y > 0 {
                                self.location.y -= 1;
                                let prev_len = self.buffer.line_at(self.location.y).len();
                                self.location.x = std::cmp::min(prev_len, self.location.x);
                            } else if self.location.x > 0 {
                                self.location.x -= 1;
                            }
                            self.update_cursor(&mut stdout)?
                        }
                        self.view.buffer = self.buffer.clone();
                    }
                    Key::Esc => {
                        self.set_mode(Mode::Normal);
                        self.update_cursor(&mut stdout)?;
                    }
                    Key::Left | Key::Right | Key::Up | Key::Down => {
                        self.handle_cursor(key, &mut stdout)?;
                    }
                    _ => {}
                },
                Mode::Command => match key {
                    Key::Esc => {
                        self.set_mode(Mode::Normal);
                        self.update_cursor(&mut stdout)?;
                    }
                    _ => {}
                },
            }
            self.view.render(&mut stdout)?;
            self.update_cursor(&mut stdout)?;
            stdout.flush().unwrap();
        }
        write!(
            stdout,
            "{}{}",
            ratatui::termion::cursor::Show,
            ratatui::termion::clear::All
        )?;
        stdout.flush()?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut editor = Editor::default();
    if let Some(file_name) = env::args().nth(1) {
        let path = PathBuf::from(&file_name);
        editor.open_file(&path)?;
    }
    editor.run()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn buffer_insert_and_delete() {
        let mut buf = Buffer::default();
        let loc = Location { x: 0, y: 0 };
        buf.insert_char(&loc, 'a');
        assert_eq!(buf.line_at(0), "a");
        assert_eq!(buf.line_count(), 1);
        let deleted = buf.delete_char(&loc);
        assert!(deleted);
        assert_eq!(buf.line_at(0), "");
    }
}
