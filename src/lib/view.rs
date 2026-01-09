use crate::buffer::Buffer;
use ratatui::termion::terminal_size;
use std::io::{Result, Write};

#[derive(Default)]
pub struct View {
    pub needs_update: bool, // still does nothing
    pub offset_y: usize,
    pub offset_x: usize,
}
impl View {
    pub fn render<W: Write>(&self, stdout: &mut W, buffer: &Buffer) -> Result<()> {
        write!(
            stdout,
            "{}{}",
            ratatui::termion::clear::All,
            ratatui::termion::cursor::Goto(1, 1)
        )?;
        let (cols, rows) = terminal_size().unwrap_or((80, 24));
        let max_cols = cols as usize;
        let max_rows = rows as usize;
        let start_line = self.offset_y;
        let end_line = usize::min(start_line + max_rows, buffer.line_count());
        for line in &buffer.lines[start_line..end_line] {
            let visible = if self.offset_x < line.len() {
                &line[self.offset_x..usize::min(self.offset_x + max_cols, line.len())]
            } else {
                ""
            };
            write!(stdout, "{}\r\n", visible)?;
        }
        stdout.flush()
    }
}
