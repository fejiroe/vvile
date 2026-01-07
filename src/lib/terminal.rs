use ratatui::termion::{
    cursor,
    raw::{IntoRawMode, RawTerminal},
    screen::{AlternateScreen, IntoAlternateScreen, ToMainScreen},
};
use std::io::{Read, Result, Stdout, Write, stdin, stdout};

pub struct Terminal {
    pub stdout: AlternateScreen<RawTerminal<Stdout>>,
}

impl Terminal {
    pub fn new() -> std::io::Result<Self> {
        let stdout = stdout().into_raw_mode()?.into_alternate_screen()?;
        Ok(Self { stdout })
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = write!(self.stdout, "{}", cursor::Show);
        let _ = self.stdout.flush();
    }
}
