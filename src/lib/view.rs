use crate::buffer::Buffer;
use std::io::{Error, Result, Write};

#[derive(Default)]
pub struct View {
    pub buffer: Buffer,
    pub needs_update: bool,
}
impl View {
    pub fn render(&self, stdout: &mut std::io::Stdout) -> Result<()> {
        write!(
            stdout,
            "{}{}",
            ratatui::termion::clear::All,
            ratatui::termion::cursor::Goto(1, 1)
        )?;

        for (idx, line) in self.buffer.lines.iter().enumerate() {
            write!(stdout, "{}\r\n", line)?;
        }
        stdout.flush()
    }
}
