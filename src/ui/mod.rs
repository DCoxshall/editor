pub mod ui_descriptor;
pub mod view_bounds;
pub mod visual_box;

use crate::{
    editor::{Editor, Mode, tab::Layout, view::View},
    terminal::Terminal,
};
use crossterm::style::Attributes;
use crossterm::style::Color;
use crossterm::style::ContentStyle;
use ui_descriptor::UiDescriptor;
use view_bounds::ViewBounds;
use visual_box::VisualBox;

pub fn render(editor: &Editor, term: &mut Terminal) {
    term.hide_cursor();
    let (width, height) = term.size();

    render_active_tab(editor, term, width, height - 1);
    render_command_bar(editor, term, 0, height - 1, width);

    term.render();
    term.show_cursor();
    term.flush();
}

fn render_active_tab(editor: &Editor, term: &mut Terminal, width: usize, height: usize) {
    let tab = editor.get_current_tab();
    let descriptor = UiDescriptor::from_tab(tab, width, height);

    for view_bounds in &descriptor.views {
        // Look up the view by path
        let view_layout = tab
            .get_layout_at(&view_bounds.path)
            .expect("Path in UiDescriptor should be valid");

        match view_layout {
            Layout::Leaf(view) => {
                let is_focused = view_bounds.path == descriptor.focused_path;
                render_view_with_bounds(view, term, view_bounds, is_focused);
            }
            Layout::Split { .. } => {
                // This shouldn't happen if UiDescriptor is constructed correctly
                unreachable!("UiDescriptor should only contain paths to Leaf views");
            }
        }
    }
}

fn render_view_with_bounds(
    view: &View,
    term: &mut Terminal,
    bounds: &ViewBounds,
    is_focused: bool,
) {
    if bounds.height < 1 || bounds.width < 1 {
        return;
    }

    let visual_box_height = bounds.height - 1;
    let mut view_vb = VisualBox::new(bounds.width, visual_box_height);
    let view_line_count = view.buffer.text.len_lines();

    // We need to leave one line free for this view's status bar.
    for visual_line_idx in 0..(visual_box_height) {
        let file_line_idx = visual_line_idx + view.start_row;
        if file_line_idx < view_line_count {
            let line_text: String = view
                .buffer
                .text
                .line(file_line_idx)
                .to_string()
                .chars()
                .skip(view.start_col)
                .collect();
            view_vb.draw(0, visual_line_idx, &line_text);
        } else {
            view_vb.draw(0, visual_line_idx, View::EMPTY_LINE_NOTATION);
        }
    }

    render_view_status_bar(
        view,
        term,
        bounds.x,
        bounds.y + bounds.height - 1,
        bounds.width,
        is_focused,
    );

    term.draw_visual_box(bounds.x, bounds.y, view_vb);
}

fn render_view_status_bar(
    view: &View,
    term: &mut Terminal,
    x: usize,
    y: usize,
    width: usize,
    is_focused: bool,
) {
    if width < 1 {
        return;
    }

    let mut status_bar_vb = VisualBox::new(width, 1);

    let mut status_bar_text = String::new();
    status_bar_text.push_str(view.file_path.to_str().unwrap());
    status_bar_text.push_str(&" ".repeat(width - status_bar_text.len()));

    let status_bar_style = match is_focused {
        true => ContentStyle {
            foreground_color: Some(Color::Black),
            background_color: Some(Color::White),
            underline_color: None,
            attributes: Attributes::default(),
        },
        false => ContentStyle {
            foreground_color: Some(Color::Reset),
            background_color: Some(Color::Reset),
            underline_color: None,
            attributes: Attributes::default(),
        },
    };

    status_bar_vb.draw_with_style(0, 0, &status_bar_text, status_bar_style);

    term.draw_visual_box(x, y, status_bar_vb);
}

fn render_command_bar(editor: &Editor, term: &mut Terminal, x: usize, y: usize, width: usize) {
    let mut command_input_vb = VisualBox::new(width, 1);

    let mut prompt_start = match editor.mode {
        Mode::Command => String::from("$ "),
        Mode::Edit => String::from("X "),
    };
    prompt_start.push_str(&editor.command_input);
    command_input_vb.draw(0, 0, &prompt_start);

    term.draw_visual_box(x, y, command_input_vb);
}
