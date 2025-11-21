mod editor;

use editor::Editor;

use std::env;
use std::path::PathBuf;
use std::{io::Result, process::exit};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: editor [filename]");
        exit(1);
    }

    let filename = match args.get(1) {
        Some(str) => str.to_owned(),
        None => String::from(""),
    };

    let path = PathBuf::from(filename);

    let mut editor = match Editor::from_path(path) {
        Ok(editor) => editor,
        Err(_) => {
            println!("Could not read file.");
            exit(1);
        }
    };

    editor.mainloop()?;

    editor.clear_terminal()?;

    Ok(())
}