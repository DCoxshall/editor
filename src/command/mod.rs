use std::path::PathBuf;

use crate::editor::{
    Editor,
    tab::Axis::{Horizontal, Vertical},
    view::View,
};

pub mod command_bar;

#[derive(Copy, Clone)]
pub enum CommandKeyword {
    Quit,
    Save,
    Hsplit,
    Vsplit,
}

pub struct Command {
    pub keyword: CommandKeyword,
    pub arguments: Vec<String>,
}

impl Command {
    /// Apply the command to the given editor.
    pub fn apply(&mut self, editor: &mut Editor) {
        match self.keyword {
            CommandKeyword::Quit => Command::quit(editor),
            CommandKeyword::Save => self.save(editor),
            CommandKeyword::Hsplit => self.hsplit(editor),
            CommandKeyword::Vsplit => self.vsplit(editor),
        }
    }

    pub fn save_no_args(editor: &mut Editor) {
        let current_view = editor.get_current_tab().get_current_focused_view();

        if current_view.file_path.is_none() {
            editor.show_warning("Error: A name is required to save.");
            return;
        }

        match current_view.save() {
            Ok(_) => editor.command_bar.clear(),
            Err(e) => editor.show_warning(&e),
        }
    }

    pub fn save_as(editor: &mut Editor, file_path: &String, force: bool) {
        let path_buf = PathBuf::from(file_path);
        let current_view = editor.get_current_tab_mut().get_current_focused_view_mut();

        if current_view.file_path.is_some() && path_buf == current_view.file_path.clone().unwrap() {
            match current_view.save() {
                Ok(_) => editor.command_bar.clear(),
                Err(e) => editor.show_warning(&e),
            }
        } else {
            if path_buf.exists() {
                if force {
                    current_view.file_path = Some(path_buf);
                    match current_view.save() {
                        Ok(_) => editor.command_bar.clear(),
                        Err(e) => editor.show_warning(&e),
                    }
                } else {
                    editor.show_warning("Warn: File already exists. Append '!' to overwrite.");
                }
            } else {
                current_view.file_path = Some(path_buf);
                match current_view.save() {
                    Ok(_) => editor.command_bar.clear(),
                    Err(e) => editor.show_warning(&e),
                }
            }
        }
    }

    pub fn save(&mut self, editor: &mut Editor) {
        match self.arguments.len() {
            0 => Command::save_no_args(editor),
            1 => Command::save_as(editor, &self.arguments[0], false),
            2 => {
                if self.arguments[1] == "!" {
                    Command::save_as(editor, &self.arguments[0], true);
                } else {
                    editor.show_warning("Usage: save [filename] [!]")
                }
            }
            _ => editor.show_warning("Usage: save [filename] [!]"),
        }
    }

    pub fn quit(editor: &mut Editor) {
        editor.quit = true;
    }

    fn vsplit(&self, editor: &mut Editor) {
        let filename = &self.arguments[0];
        let pathbut = PathBuf::from(filename);
        let new_view = View::from_path(pathbut).unwrap();
        editor
            .get_current_tab_mut()
            .insert_new_view(new_view, Vertical);
    }

    fn hsplit(&self, editor: &mut Editor) {
        let filename = &self.arguments[0];
        let pathbuf = PathBuf::from(filename);
        let new_view = View::from_path(pathbuf).unwrap();
        editor
            .get_current_tab_mut()
            .insert_new_view(new_view, Horizontal);
    }
}
