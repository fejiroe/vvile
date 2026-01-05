use std::io::{self, Write, stdout, Read, Result, Error};
use ratatui::termion::raw::IntoRawMode;

fn main() -> Result<(),> {
    let mut stdout = stdout().into_raw_mode()?;
    for b in io::stdin().bytes() {
        let c = b.unwrap() as char;
        println!("{}", c);
    }
    Ok(())
}
