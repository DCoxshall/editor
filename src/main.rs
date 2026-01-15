mod editor;
mod terminal;
mod ui;

use editor::Editor;
use terminal::Terminal;

use std::env;
use std::io::Result;
use std::path::PathBuf;
use std::str::FromStr;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut paths: Vec<PathBuf> = Vec::new();

    for i in 1..args.len() {
        let filename = args.get(i);
        match filename {
            // unwrap justified because PathBuf::from_str is Infallible.
            Some(s) => paths.push(PathBuf::from_str(s).unwrap()),
            None => {}
        }
    }

    let mut editor = Editor::from_paths(paths);
    let mut terminal = Terminal::new()?;

    loop {
        ui::render(&editor, &mut terminal);
        let event = terminal.read_event()?;

        if event.is_resize() {
            terminal.resize();
        } else {
            editor.handle_input(event, &terminal);
        }
        if editor.quit {
            break;
        }
    }

    Ok(())
}
