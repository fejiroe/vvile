use crate::buffer::Buffer;
use ratatui::termion::terminal_size;
use std::io::{Result, Write};

#[derive(Default, Debug)]
pub struct View {
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
        let end_line = start_line.saturating_add(max_rows).min(buffer.line_count());
        for line in &buffer.lines[start_line..end_line] {
            let start_grapheme = self.offset_x.min(line.grapheme_len());
            let end_grapheme = usize::min(start_grapheme + max_cols, line.grapheme_len());
            let start_byte = *line
                .graphemes
                .get(start_grapheme)
                .unwrap_or(&line.raw.len());
            let end_byte = *line.graphemes.get(end_grapheme).unwrap_or(&line.raw.len());
            let visible = &line.raw[start_byte..end_byte];
            write!(stdout, "{}\r\n", visible)?;
        }
        stdout.flush()
    }
}
