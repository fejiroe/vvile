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
        let max_y = self.buffer.line_count();
        debug_assert!(
            self.cursor.y < max_y,
            "Cursor y out of bounds: {} >= {}",
            self.cursor.y,
            max_y
        );
        let line_len = if self.cursor.y < max_y {
            self.buffer.lines[self.cursor.y].grapheme_len()
        } else {
            0
        };
        debug_assert!(
            self.cursor.x <= line_len,
            "Cursor x out of bounds: {} > {}",
            self.cursor.x,
            line_len
        );
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::{Line, Location};
    use std::fs;
    use std::path::PathBuf;

    // Helper to create a temporary file path
    fn temp_file_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(name)
    }

    #[test]
    fn test_open_nonexistent_file() {
        let mut editor = Editor::default();
        let path = temp_file_path("test_nonexistent_editor.txt");
        if path.exists() {
            fs::remove_file(&path).unwrap();
        }
        editor.open_file(&path).expect("open_file");
        assert_eq!(editor.current_file().as_ref(), Some(&path));
        assert_eq!(editor.buffer.line_count(), 1);
        assert_eq!(editor.buffer.lines[0].grapheme_len(), 0);
    }

    #[test]
    fn test_write_file() {
        let mut editor = Editor::default();
        // Insert "hello" into buffer
        for (i, c) in "hello".chars().enumerate() {
            editor.buffer.insert_char(&Location { x: i, y: 0 }, c);
        }
        let path = temp_file_path("test_write_editor.txt");
        if path.exists() {
            fs::remove_file(&path).unwrap();
        }
        editor.write_file(&path).expect("write_file");
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "hello");
    }

    #[test]
    fn test_delete_under_cursor_removes_character() {
        let mut editor = Editor::default();
        editor.buffer.lines[0] = Line::from_string("abc".to_owned());
        editor
            .buffer
            .lines
            .push(Line::from_string("def".to_owned()));
        editor.cursor.y = 0;
        editor.cursor.x = 1; // on 'b'
        editor.delete_under_cursor();
        assert_eq!(editor.buffer.lines[0].as_str(), "ac");
        assert_eq!(editor.buffer.line_count(), 2);
    }

    #[test]
    fn test_delete_under_cursor_merges_line_and_next() {
        let mut editor = Editor::default();
        editor.buffer.lines[0] = Line::from_string("abc".to_owned());
        editor
            .buffer
            .lines
            .push(Line::from_string("def".to_owned()));
        editor.cursor.y = 0;
        editor.cursor.x = 3; // at end of line
        editor.delete_under_cursor();
        assert_eq!(editor.buffer.line_count(), 1);
        assert_eq!(editor.buffer.lines[0].as_str(), "abcdef");
    }

    #[test]
    fn test_open_existing_file() {
        let path = temp_file_path("test_open.txt");
        let content = "line1\nline2";
        fs::write(&path, content).unwrap();
        let mut editor = Editor::default();
        editor.open_file(&path).expect("open_existing");
        assert_eq!(editor.buffer.line_count(), 2);
        assert_eq!(editor.buffer.lines[0].as_str(), "line1");
        assert_eq!(editor.buffer.lines[1].as_str(), "line2");
    }
}
