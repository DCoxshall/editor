pub mod visual_box;

use crossterm::style::Attributes;
use crossterm::style::Color;
use crossterm::style::ContentStyle;

use crate::editor::Mode;
use crate::ui::visual_box::VisualBox;
use crate::{editor::Editor, terminal::Terminal};

use crate::editor::tab::{Axis, Layout};
use crate::editor::view::View;

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
    render_layout(editor, &tab.layout, term, 0, 0, width, height);
}

/// Recursively render the nested layout enum. This function is not as complicated as it looks -
/// recursively, if we're at a Leaf variant then we render the contained view, and if we're at a
/// Split variant, we determine the sizes of the upper and lower/left and right splits, and render
/// them.
fn render_layout(
    editor: &Editor,
    layout: &Layout,
    term: &mut Terminal,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) {
    match layout {
        Layout::Leaf(view) => {
            render_view(editor, view, term, x, y, width, height);
        }
        Layout::Split {
            axis,
            weight,
            children,
        } => match axis {
            Axis::Vertical => {
                let left_width = (width as f32 * weight) as usize;
                let mut right_width = (width as f32 - (width as f32 * weight)) as usize;

                // Avoid off-by-one errors when applying split weights.
                if left_width + right_width != width {
                    right_width += 1;
                }
                let right_x = x + left_width;
                render_layout(editor, &children[0], term, x, y, left_width, height);
                render_layout(editor, &children[1], term, right_x, y, right_width, height);
            }
            Axis::Horizontal => {
                let upper_height = (height as f32 * weight) as usize;
                let mut lower_height = (height as f32 - (height as f32 * weight)) as usize;

                // Avoid off-by-one errors when applying split weights.
                if upper_height + lower_height != height {
                    lower_height += 1;
                }
                let lower_y = y + upper_height;
                render_layout(editor, &children[0], term, x, y, width, upper_height);
                render_layout(editor, &children[1], term, x, lower_y, width, lower_height);
            }
        },
    }
}

fn render_view(
    editor: &Editor,
    view: &View,
    term: &mut Terminal,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) {
    if height < 1 || width < 1 {
        return;
    }

    let visual_box_height = height - 1;
    let mut view_vb = VisualBox::new(width, visual_box_height);
    let view_line_count = view.buffer.text.len_lines();

    // We need to leave one line free for this view's status bar.
    for visual_line_idx in 0..(visual_box_height) {
        if visual_line_idx + view.start_row < view_line_count {
            let line_text: String = view
                .buffer
                .text
                .line(visual_line_idx + view.start_row)
                .to_string()
                .chars()
                .skip(view.start_col)
                .collect();
            view_vb.draw(0, visual_line_idx, &line_text);
        } else {
            view_vb.draw(0, visual_line_idx, View::EMPTY_LINE_NOTATION);
        }
    }

    render_view_status_bar(editor, view, term, x, y + height - 1, width);

    term.draw_visual_box(x, y, view_vb);
}

fn render_view_status_bar(
    editor: &Editor,
    view: &View,
    term: &mut Terminal,
    x: usize,
    y: usize,
    width: usize,
) {
    if width < 1 {
        return;
    }

    let mut status_bar_vb = VisualBox::new(width, 1);

    let mut status_bar_text = String::new();
    status_bar_text.push_str(view.file_path.to_str().unwrap());
    status_bar_text.push_str(&" ".repeat(width - status_bar_text.len()));

    let focused_view_ref = editor.get_current_tab().get_current_focused_view();

    let is_focused = std::ptr::eq(focused_view_ref, view);

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
