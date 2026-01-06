use std::io::{self, Write, stdout, Read, Result, Error};
use ratatui::termion::raw::IntoRawMode;

struct Editor {
}

impl Editor {
    fn default() -> Self {Editor{}}
    fn run(&self) -> Result<(), > {
        let mut stdout = stdout().into_raw_mode()?;
        for b in io::stdin().bytes() {
            let c = b.unwrap() as char;
            println!("{}", c);
            if c == 'q' {
                break;
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), > {
    let editor = Editor::default();
    editor.run();
    Ok(())
}
