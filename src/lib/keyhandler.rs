use ratatui::termion::event::Key;
use std::io::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Edit,
    Command,
    Visual,
}

pub struct KeyHandler<'a> {
    editor: &'a mut super::Editor,
}

impl<'a> KeyHandler<'a> {
    pub fn new(editor: &'a mut super::Editor) -> Self {
        Self { editor }
    }

    pub fn process_key(&mut self, key: Key, stdout: &mut std::io::Stdout) -> Result<()> {
        match self.editor.mode {
            Mode::Normal => self.handle_normal(key, stdout),
            Mode::Edit => self.handle_edit(key, stdout),
            Mode::Command => self.handle_command(key, stdout),
            Mode::Visual => self.handle_visual(key, stdout),
        }
    }
    fn handle_normal(&mut self, key: Key, stdout: &mut std::io::Stdout) -> Result<()> {
        match key {
            Key::Char(':') => self.editor.set_mode(Mode::Command),
            Key::Char('a') => {
                let line_len = self.editor.buffer.line_at(self.editor.cursor.y).len();
                if self.editor.cursor.x < line_len {
                    self.editor.cursor.x += 1;
                } else if self.editor.cursor.y + 1 < self.editor.buffer.line_count() {
                    self.editor.cursor.y += 1;
                    self.editor.cursor.x = 0;
                }
                self.editor.set_mode(Mode::Edit);
            }
            Key::Char('i') => {
                self.editor.set_mode(Mode::Edit);
            }
            Key::Char('x') => {
                self.editor.delete_under_cursor();
                self.editor.update_view();
                self.editor.update_cursor(stdout)?;
            }
            Key::Char('s') => {
                self.editor.delete_under_cursor();
                self.editor.update_view();
                self.editor.update_cursor(stdout)?;
                self.editor.set_mode(Mode::Edit);
            }
            Key::Char('v') => {
                self.editor.set_mode(Mode::Visual);
            }
            Key::Left | Key::Right | Key::Up | Key::Down => {
                self.editor.handle_cursor(key)?;
                self.editor.update_view();
                self.editor.update_cursor(stdout)?;
            }
            Key::Ctrl('s') => {
                self.editor.write_file(&self.editor.current_file)?;
            }
            Key::Ctrl('q') => {
                std::process::exit(0);
            }
            _ => {}
        }
        Ok(())
    }
    fn handle_edit(&mut self, key: Key, stdout: &mut std::io::Stdout) -> Result<()> {
        match key {
            Key::Char('\n') => {
                let line = self.editor.buffer.line_at(self.editor.cursor.y).to_owned();
                let byte_offset =
                    self.editor.buffer.lines[self.editor.cursor.y].graphemes[self.editor.cursor.x];
                let (left, right) = line.split_at(byte_offset);
                self.editor.buffer.lines[self.editor.cursor.y] =
                    crate::buffer::Line::from_string(left.to_owned());
                self.editor.buffer.lines.insert(
                    self.editor.cursor.y + 1,
                    crate::buffer::Line::from_string(right.to_owned()),
                );
                self.editor.cursor.y += 1;
                self.editor.cursor.x = 0;
                self.editor.update_view();
                self.editor.update_cursor(stdout)?;
            }
            Key::Char('\t') => {
                let tab_width = 4;
                let target_col = (self.editor.cursor.x / tab_width + 1) * tab_width;
                let spaces_needed = target_col - self.editor.cursor.x;
                for _ in 0..spaces_needed {
                    self.editor
                        .buffer
                        .insert_char(&(crate::buffer::Location::from(self.editor.cursor)), ' ');
                }
                self.editor.cursor.x = target_col;
                self.editor.update_view();
                self.editor.update_cursor(stdout)?;
            }
            Key::Char(c) => {
                self.editor
                    .buffer
                    .insert_char(&(crate::buffer::Location::from(self.editor.cursor)), c);
                self.editor.cursor.x += 1;
                self.editor.update_view();
                self.editor.update_cursor(stdout)?;
            }
            Key::Backspace => {
                self.editor.delete_under_cursor();
                if self.editor.cursor.x == 0 && self.editor.cursor.y > 0 {
                    self.editor.cursor.y -= 1;
                    let prev_len = self.editor.buffer.lines[self.editor.cursor.y].grapheme_len();
                    self.editor.cursor.x = std::cmp::min(prev_len, self.editor.cursor.x);
                } else if self.editor.cursor.x > 0 {
                    self.editor.cursor.x -= 1;
                }
                self.editor.update_view();
                self.editor.update_cursor(stdout)?;
            }
            Key::Esc => {
                self.editor.set_mode(Mode::Normal);
                self.editor.update_view();
                self.editor.update_cursor(stdout)?;
            }
            Key::Left | Key::Right | Key::Up | Key::Down => {
                self.editor.handle_cursor(key)?;
                self.editor.update_view();
                self.editor.update_cursor(stdout)?;
            }
            _ => {}
        }

        Ok(())
    }
    fn handle_command(&mut self, key: Key, stdout: &mut std::io::Stdout) -> Result<()> {
        match key {
            Key::Esc => {
                self.editor.set_mode(Mode::Normal);
                self.editor.update_view();
                self.editor.update_cursor(stdout)?;
            }
            _ => {}
        }
        Ok(())
    }
    fn handle_visual(&mut self, key: Key, stdout: &mut std::io::Stdout) -> Result<()> {
        match key {
            Key::Esc => {
                self.editor.set_mode(Mode::Normal);
                self.editor.update_view();
                self.editor.update_cursor(stdout)?;
            }
            _ => {}
        }
        Ok(())
    }
}
