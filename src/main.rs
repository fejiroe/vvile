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

struct Editor {
    mode: Mode,
    location: Location,
    max_cols: usize,
    max_rows: usize,
}

impl Editor {
    fn new() -> Self {
        let (cols, rows) = ratatui::termion::terminal_size().unwrap_or((80, 24));
        Self {
            mode: Mode::Normal,
            location: Location { x: 0, y: 0 },
            max_cols: cols as usize,
            max_rows: rows as usize,
        }
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
        if self.location.x > 0 {
            self.location.x -= 1;
        }
        self.update_cursor(stdout)?;
        Ok(())
    }
    fn move_right(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        if self.location.x + 1 < self.max_cols {
            self.location.x += 1;
        }
        self.update_cursor(stdout)?;
        Ok(())
    }
    fn move_up(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        if self.location.y > 0 {
            self.location.y -= 1;
        }
        self.update_cursor(stdout)?;
        Ok(())
    }
    fn move_down(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        if self.location.y + 1 < self.max_rows {
            self.location.y += 1;
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
                        write!(stdout, "{}", c)?;
                    }
                    Key::Left | Key::Right | Key::Up | Key::Down => {
                        self.handle_cursor(key, &mut stdout)?
                    }
                    Key::Backspace => {
                        self.move_left(&mut stdout)?;
                        write!(stdout, "\x08 \x08")?;
                    }
                    Key::Esc => {
                        self.set_mode(Mode::Normal);
                    }
                    _ => {}
                },
            }
            stdout.flush().unwrap();
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    Editor::new().run();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_something() {}
}
