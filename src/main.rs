use std::io::{self, Write, stdin, stdout, Read, Result, Error};
use ratatui::termion::raw::IntoRawMode;
use ratatui::termion::event::{Key, Event};
use ratatui::termion::input::TermRead;

struct Editor {
}

impl Editor {
    fn default() -> Self {Editor{}}
    fn run(&self) -> Result<(), > {
        let stdin = stdin();
        let mut stdout = stdout().into_raw_mode()?;
        write!(stdout,
            "{}{}ctrl q to exit",
            ratatui::termion::clear::All,
            ratatui::termion::cursor::Goto(1,1)
            ).unwrap();
        stdout.flush().unwrap();
        for k in stdin.keys() {
            let key = k.unwrap();
            match key {
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
}

fn main() -> Result<(), > {
    Editor::default().run();
    Ok(())
}
