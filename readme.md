simple modal editor in rust.

'i' for insert mode, 'esc' to return to normal mode, 'ctrl+q' to exit, 'ctrl+s' to save the file in normal mode.

"cargo build --release" and copy wherever appropriate or "cargo install --path ."

todo:

- proper viewport: infinite scroll, handle files that don't fit on screen
- terminal size
- undo/redo
- about / help / man screen/popup
- more motions
- cmds
- graphemes
- visual/select mode
- config
- status line (+ print dbg to status line)
- syntax highlighting

- bugs, error handling, testing

secret feature?
