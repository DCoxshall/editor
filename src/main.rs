mod editor;

use editor::Editor;

use crossterm::{
    cursor::{MoveTo, Show},
    event::{Event, read},
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};

use std::cmp::max;
use std::env;
use std::fs;
use std::io::{Stdout, Write, stdout};
use std::{io::Result, process::exit};

fn clear_terminal(stdout: &mut Stdout) -> Result<()> {
    execute!(stdout, Clear(ClearType::All))?;
    execute!(stdout, MoveTo(0, 0))?;
    stdout.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: editor [filename]");
        exit(1);
    }

    let filename = args.get(1).unwrap();
    let contents = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(_) => {
            println!("Could not read file.");
            exit(1);
        }
    };

    let mut editor = Editor::from_string(contents, filename.to_owned());

    let mut stdout = stdout();

    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;

    loop {
        editor.render(&mut stdout)?;
        stdout.flush()?;
        match read()? {
            Event::Key(key_event) => {
                let quit = editor.handle_key_event(key_event);
                if quit {
                    break;
                }
            }
            Event::Resize(w, h) => {
                editor.buffer.visual_width = w as usize;
                editor.buffer.visual_height = max(h, 0) as usize;
            }
            _ => {}
        }

        // After every input event, we need to ensure that the cursor remains on screen.
        editor.align_cursor();
    }

    clear_terminal(&mut stdout)?;

    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen, Show)?;
    Ok(())
}
