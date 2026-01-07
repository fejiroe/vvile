use ratatui::termion::event::Key;
use ratatui::termion::input::TermRead;
use std::io::ErrorKind;
use std::io::{Error, Read, Result, Write, stdin, stdout};
use std::path::{Path, PathBuf};

use crate::buffer::Buffer;
use crate::buffer::Location;
use crate::terminal::Terminal;
use crate::view::View;

enum Mode {
    Normal,
    Edit,
    Command,
}

pub struct Editor {
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
    pub fn open_file(&mut self, at: &Path) -> Result<()> {
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
    pub fn run(&mut self) -> Result<()> {
        let stdin = stdin();
        let mut term = Terminal::new()?;
        write!(
            term.stdout,
            "{}{}",
            ratatui::termion::clear::All,
            ratatui::termion::cursor::Goto(1, 1)
        )
        .unwrap();
        term.stdout.flush().unwrap();
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
                        self.handle_cursor(key, &mut term.stdout)?
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
                        self.update_cursor(&mut term.stdout)?;
                    }
                    Key::Char('\t') => {
                        let tab_width = 4;
                        for _ in 0..tab_width {
                            self.buffer.insert_char(&self.location, ' ');
                        }
                        let target_col =
                            ((self.location.x + tab_width - 1) / tab_width) * tab_width;
                        self.location.x = target_col;
                        self.update_cursor(&mut term.stdout)?;
                        self.view.buffer = self.buffer.clone();
                    }
                    Key::Char(c) => {
                        self.buffer.insert_char(&self.location, c);
                        self.location.x += 1;
                        self.update_cursor(&mut term.stdout)?;
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
                            self.update_cursor(&mut term.stdout)?
                        }
                        self.view.buffer = self.buffer.clone();
                    }
                    Key::Esc => {
                        self.set_mode(Mode::Normal);
                        self.update_cursor(&mut term.stdout)?;
                    }
                    Key::Left | Key::Right | Key::Up | Key::Down => {
                        self.handle_cursor(key, &mut term.stdout)?;
                    }
                    _ => {}
                },
                Mode::Command => match key {
                    Key::Esc => {
                        self.set_mode(Mode::Normal);
                        self.update_cursor(&mut term.stdout)?;
                    }
                    _ => {}
                },
            }
            self.view.render(&mut term.stdout)?;
            self.update_cursor(&mut term.stdout)?;
            term.stdout.flush().unwrap();
        }
        Ok(())
    }
}
