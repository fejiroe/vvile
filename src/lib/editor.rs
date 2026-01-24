use ratatui::termion::event::Key;
use ratatui::termion::input::TermRead;
use std::fs;
use std::io::ErrorKind;
use std::io::{Result, Write, stdin};
use std::path::{Path, PathBuf};

use crate::buffer::Buffer;
use crate::cursor::Cursor;
use crate::keyhandler::{KeyHandler, Mode};
use crate::terminal::Terminal;
use crate::view::View;

#[derive(Debug, Default)]
pub struct Editor {
    current_file: Option<PathBuf>,
    mode: Mode,
    pub buffer: Buffer,
    pub view: View,
    pub cursor: Cursor,
}

impl Editor {
    pub fn current_file(&self) -> &Option<PathBuf> {
        &self.current_file
    }
    pub fn opened_file(&self) -> Option<&Path> {
        self.current_file.as_ref().map(|pb| pb.as_path())
    }
    pub fn open_file(&mut self, at: &Path) -> Result<()> {
        self.current_file = Some(at.to_path_buf());
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
        fs::write(path, self.buffer.buffer_to_string())?;
        Ok(())
    }
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
    pub fn get_mode(&self) -> Mode {
        self.mode
    }
    pub fn update_view(&mut self) {
        let (cols, rows) = ratatui::termion::terminal_size().unwrap_or((80, 24));
        let (_max_cols, max_rows) = (cols as usize, rows as usize);
        if self.buffer.line_count() == 0 {
            self.view.offset_x = 0;
            self.view.offset_y = 0;
            return;
        }
        let (new_offset_x, new_offset_y) = self.cursor.maybe_scroll(&self.view);
        let line = &self.buffer.lines[new_offset_y];
        self.view.offset_x = new_offset_x.min(line.grapheme_len());
        let max_offset_y = self.buffer.line_count().saturating_sub(max_rows);
        self.view.offset_y = new_offset_y.min(max_offset_y);
    }
    pub fn update_cursor(&self, stdout: &mut std::io::Stdout) -> Result<()> {
        self.cursor
            .render_cursor(self.view.offset_x, self.view.offset_y, stdout)
    }
    pub fn handle_cursor(&mut self, key: Key) -> Result<()> {
        match key {
            Key::Left => self.cursor.move_left(&self.buffer),
            Key::Right => self.cursor.move_right(&self.buffer),
            Key::Up => self.cursor.move_up(&self.buffer),
            Key::Down => self.cursor.move_down(&self.buffer),
            _ => {}
        }
        Ok(())
    }
    pub fn delete_under_cursor(&mut self) {
        let line_len = self.buffer.lines[self.cursor.y].grapheme_len();
        if self.cursor.x < line_len {
            let line = &mut self.buffer.lines[self.cursor.y];
            line.remove(self.cursor.x);
        } else if self.cursor.y + 1 < self.buffer.line_count() {
            let next = self.buffer.lines.remove(self.cursor.y + 1);
            self.buffer.lines[self.cursor.y].push_str(&next.raw);
        }
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
        let stdin = stdin();
        for k in stdin.keys() {
            let key = k?;
            let mut kh = KeyHandler::new(self);
            kh.process_key(key, &mut term.stdout)?;
            self.view.render(&mut term.stdout, &self.buffer)?;
            self.update_cursor(&mut term.stdout)?;
            term.stdout.flush().unwrap();
        }
        Ok(())
    }
}
