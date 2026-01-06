use std::io::{stdin, stdout, Result, Error, Read, Write};
use ratatui::termion::raw::IntoRawMode;
use ratatui::termion::input::TermRead;
use ratatui::termion::event::Key;
use ratatui::termion::terminal_size;
use std::env;

struct Editor {
}

impl Editor {
    fn default() -> Self {Editor{}}
    fn run(&self) -> Result<(),> {
        // let mut size = terminal_size().unwrap();
        let stdin = stdin();
        let mut stdout = stdout().into_raw_mode()?;
        write!(stdout,
            "{}{}",
            ratatui::termion::clear::All,
            ratatui::termion::cursor::Goto(1,1)
            ).unwrap();
        stdout.flush().unwrap();
        for k in stdin.keys() {
            let key = k.unwrap();
            match key {
                Key::Char('a') => Self::draw_rows(&self)?,
                // Key::Char('i') => ,
                // Key::Char('x') => ,
                // Key::Char('s') => ,
                // Key::Char('r') => ,
                // Key::Char('v') => ,
                Key::Ctrl(c) => match c {
                    'q' => break, 
                    _ => {}
                }
                _ => {}
            }
            stdout.flush().unwrap();
        }
        Ok(())
    }
    fn draw_rows(&self) -> Result<(),> {
        let height = terminal_size()?.1;
        for current_row in 0..height {
            print!("~");
            if current_row + 1 < height {
                print!("\r\n")
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), > {
    Editor::default().run();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_something() {}
}
