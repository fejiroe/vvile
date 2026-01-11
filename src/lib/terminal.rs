use ratatui::termion::{
    cursor,
    raw::{IntoRawMode, RawTerminal},
    screen::{ToAlternateScreen, ToMainScreen},
};
use std::io::{Stdout, Write, stdout};

pub struct Terminal {
    pub stdout: RawTerminal<Stdout>,
}

impl Terminal {
    pub fn new() -> std::io::Result<Self> {
        let mut stdout = stdout().into_raw_mode()?;
        write!(stdout, "{}", ToAlternateScreen)?;
        Ok(Self { stdout })
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = write!(stdout(), "{}", ToMainScreen);
        let _ = write!(self.stdout, "{}", cursor::Show);
        let _ = self.stdout.flush();
    }
}
