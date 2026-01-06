use std::io::{self, Write, stdout, Read, Result, Error};
use ratatui::termion::raw::IntoRawMode;
use ratatui::termion::event::{Key, Event};
use ratatui::termion::input::TermRead;

struct Editor {
}

impl Editor {
    fn default() -> Self {Editor{}}
    fn run(&self) -> Result<(), > {
        let mut stdout = stdout().into_raw_mode()?;
        write!(stdout, "{}{}q to exit", termion::clear::All, termion::cursor::Goto(1,1)).unwrap();
        stdout.flush().unwrap();
        /*
        for b in io::stdin().bytes() {
            let c = b.unwrap() as char;
            println!("{}", c);
            if c == 'q' {
                break;
            }
        }
        */
        for c in stdin.events() {
            let evt = c.unwrap();
            match evt {
                Event::Key(Key::char('q')) => break;
            }
            _ => {}
            stdout.flush().unwrap();
        }
        Ok(())
    }
}

fn main() -> Result<(), > {
    let editor = Editor::default();
    editor.run();
    Ok(())
}
