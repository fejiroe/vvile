use ratatui::termion::event::Key;
use ratatui::termion::input::TermRead;
use std::fs;
use std::io::ErrorKind;
use std::io::{Result, Write, stdin};
use std::path::{Path, PathBuf};

use crate::buffer::Buffer;
use crate::buffer::Line;
use crate::buffer::Location;
use crate::cursor::Cursor;
use crate::terminal::Terminal;
use crate::view::View;

enum Mode {
    Normal,
    Edit,
    Command,
    Visual,
}

pub struct Editor {
    current_file: PathBuf,
    mode: Mode,
    pub buffer: Buffer,
    view: View,
    cursor: Cursor,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            current_file: PathBuf::new(),
            mode: Mode::Normal,
            buffer: Buffer::default(),
            view: View {
                offset_y: 0,
                offset_x: 0,
                ..Default::default()
            },
            cursor: Cursor::default(),
        }
    }
}

impl Editor {
    pub fn open_file(&mut self, at: &Path) -> Result<()> {
        self.current_file = at.to_path_buf();
        match self.buffer.read_file(at) {
            Ok(()) => {}
            Err(e) if e.kind() == ErrorKind::NotFound => {
                self.buffer = Buffer::default();
            }
            Err(e) => return Err(e),
        }
        self.view.offset_x = 0;
        self.view.offset_y = 0;
        Ok(())
    }
    pub fn write_file(&self, path: &Path) -> Result<()> {
        let out = self.buffer.buffer_to_string();
        fs::write(path, out)?;
        Ok(())
    }
    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
    fn update_view(&mut self) {
        let (cols, rows) = ratatui::termion::terminal_size().unwrap_or((80, 24));
        let _max_cols = cols as usize;
        let max_rows = rows as usize;
        let (new_offset_x, new_offset_y) = self.cursor.maybe_scroll(&self.view);
        let line = &self.buffer.lines[new_offset_y];
        let current_line_len = line.grapheme_len();
        self.view.offset_x = new_offset_x.min(current_line_len);
        let max_offset_y = self.buffer.line_count().saturating_sub(max_rows);
        self.view.offset_y = new_offset_y.min(max_offset_y);
    }
    fn update_cursor(&self, stdout: &mut std::io::Stdout) -> Result<()> {
        self.cursor
            .render_cursor(self.view.offset_x, self.view.offset_y, stdout)
    }
    fn handle_cursor(&mut self, key: Key) -> Result<()> {
        match key {
            Key::Left => self.cursor.move_left(&self.buffer),
            Key::Right => self.cursor.move_right(&self.buffer),
            Key::Up => self.cursor.move_up(&self.buffer),
            Key::Down => self.cursor.move_down(&self.buffer),
            _ => {}
        }
        Ok(())
    }
    fn delete_under_cursor(&mut self) {
        let line_len = self.buffer.lines[self.cursor.y].grapheme_len();
        if self.cursor.x < line_len {
            let line = &mut self.buffer.lines[self.cursor.y];
            line.remove(self.cursor.x);
        } else if self.cursor.y + 1 < self.buffer.line_count() {
            let next = self.buffer.lines.remove(self.cursor.y + 1);
            self.buffer.lines[self.cursor.y].push_str(&next.raw);
        }
    }
    fn handle_keys(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        let stdin = stdin();
        for k in stdin.keys() {
            let key = k?;
            match self.mode {
                Mode::Normal => match key {
                    Key::Char(':') => self.set_mode(Mode::Command),
                    Key::Char('a') => {
                        let line_len = self.buffer.line_at(self.cursor.y).len();
                        if self.cursor.x < line_len {
                            self.cursor.x += 1;
                        } else if self.cursor.y + 1 < self.buffer.line_count() {
                            self.cursor.y += 1;
                            self.cursor.x = 0;
                        }
                        self.set_mode(Mode::Edit);
                    }
                    Key::Char('i') => self.set_mode(Mode::Edit),
                    Key::Char('x') => {
                        self.delete_under_cursor();
                        self.update_view();
                        self.update_cursor(stdout)?;
                    }
                    Key::Char('s') => {
                        self.delete_under_cursor();
                        self.update_view();
                        self.update_cursor(stdout)?;
                        self.set_mode(Mode::Edit);
                    }
                    // Key::Char('b') =>
                    // Key::Char('w') =>
                    // Key::Char('e') =>
                    // Key::Char('r') =>
                    // Key::Char('u') => ,
                    // Key::Char('/') => ,
                    // Key::Char('?') => ,
                    Key::Char('v') => self.set_mode(Mode::Visual),
                    Key::Left | Key::Right | Key::Up | Key::Down => {
                        self.handle_cursor(key)?;
                        self.update_view();
                        self.update_cursor(stdout)?;
                    }
                    Key::Ctrl('s') => self.write_file(&self.current_file)?,
                    Key::Ctrl('q') => break,
                    _ => {}
                },
                Mode::Edit => match key {
                    Key::Char('\n') => {
                        let cur_line = self.buffer.line_at(self.cursor.y).to_owned();
                        let (left, right) = cur_line.split_at(self.cursor.x);
                        self.buffer.lines[self.cursor.y] = Line::from_string(left.to_owned());
                        self.buffer
                            .lines
                            .insert(self.cursor.y + 1, Line::from_string(right.to_owned()));
                        self.cursor.y += 1;
                        self.cursor.x = 0;
                        self.update_view();
                        self.update_cursor(stdout)?;
                    }
                    Key::Char('\t') => {
                        let tab_width = 4;
                        let target_col = (self.cursor.x / tab_width + 1) * tab_width;
                        let spaces_needed = target_col - self.cursor.x;
                        for _ in 0..spaces_needed {
                            self.buffer.insert_char(&(Location::from(self.cursor)), ' ');
                        }
                        self.cursor.x = target_col;
                        self.update_view();
                        self.update_cursor(stdout)?;
                    }
                    Key::Char(c) => {
                        self.buffer.insert_char(&(Location::from(self.cursor)), c);
                        self.cursor.x += 1;
                        self.update_view();
                        self.update_cursor(stdout)?;
                    }
                    Key::Backspace => {
                        self.delete_under_cursor();
                        if self.cursor.x == 0 && self.cursor.y > 0 {
                            self.cursor.y -= 1;
                            let prev_len = self.buffer.lines[self.cursor.y].grapheme_len();
                            self.cursor.x = std::cmp::min(prev_len, self.cursor.x);
                        } else if self.cursor.x > 0 {
                            self.cursor.x -= 1;
                        }
                        self.update_view();
                        self.update_cursor(stdout)?
                    }
                    Key::Esc => {
                        self.set_mode(Mode::Normal);
                        self.update_view();
                        self.update_cursor(stdout)?;
                    }
                    Key::Left | Key::Right | Key::Up | Key::Down => {
                        self.handle_cursor(key)?;
                        self.update_view();
                        self.update_cursor(stdout)?;
                    }
                    _ => {}
                },
                Mode::Command => match key {
                    Key::Esc => {
                        self.set_mode(Mode::Normal);
                        self.update_view();
                        self.update_cursor(stdout)?;
                    }
                    _ => {}
                },
                Mode::Visual => match key {
                    Key::Esc => {
                        self.set_mode(Mode::Normal);
                        self.update_view();
                        self.update_cursor(stdout)?;
                    }
                    _ => {}
                },
            }
            self.view.render(stdout, &self.buffer)?;
            self.update_cursor(stdout)?;
            stdout.flush().unwrap();
        }
        Ok(())
    }
    pub fn run(&mut self) -> Result<()> {
        let mut term = Terminal::new()?;
        write!(
            term.stdout,
            "{}{}",
            ratatui::termion::clear::All,
            ratatui::termion::cursor::Goto(1, 1)
        )
        .unwrap();
        term.stdout.flush().unwrap();
        self.view.render(&mut term.stdout, &self.buffer)?;
        self.update_cursor(&mut term.stdout)?;
        self.handle_keys(&mut term.stdout)?;
        Ok(())
    }
}
