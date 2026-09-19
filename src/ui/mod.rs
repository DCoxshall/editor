pub mod ui_descriptor;
pub mod view_bounds;
pub mod visual_box;

use ui_descriptor::UiDescriptor;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use view_bounds::ViewBounds;
use visual_box::VisualBox;

use crate::{
    editor::{
        Editor, Mode,
        tab::{Layout, Tab},
        view::View,
    },
    terminal::Terminal,
    ui::visual_box::reset_style,
};

use crossterm::style::{Attributes, Color, ContentStyle};

pub fn render(editor: &Editor, term: &mut Terminal) {
    term.hide_cursor();

    let (width, height) = Terminal::size();
    let current_tab = editor.get_current_tab();

    let descriptor = UiDescriptor::from_tab(current_tab, width, height - 1);

    let (cursor_x, cursor_y) = get_visual_cursor_pos(editor, &descriptor, current_tab, height);

    render_active_tab(current_tab, &descriptor, term);
    render_command_bar(editor, term, 0, height - 1, width);

    term.render();
    term.place_cursor(cursor_x, cursor_y);
    term.show_cursor();
    term.flush();
}

fn render_active_tab(tab: &Tab, descriptor: &UiDescriptor, term: &mut Terminal) {
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
    // The visual X and Y coordinates of the grapheme we're currently rendering.
    let mut visual_x: usize;
    let mut visual_y: usize = 0;

    // We leave a line free for the status bar.
    let mut view_vb = VisualBox::new(bounds.width, bounds.height - 1);

    let lines = view.buffer.text.lines();

    let relevant_lines = lines.skip(view.start_row);

    for line in relevant_lines {
        visual_x = 0;
        let mut text = line.to_string();
        if text.ends_with('\n') {
            text.pop();
        }
        if text.ends_with('\r') {
            text.pop();
        }
        let graphemes = text.graphemes(true);
        for g in graphemes {
            view_vb.draw_grapheme(visual_x, visual_y, g, 0, reset_style());
            visual_x += g.width_cjk();
            if visual_x >= view_vb.width {
                break;
            }
        }
        visual_y += 1;
        if visual_y >= view_vb.height {
            break;
        }
    }

    // Fill in the rest of the visual box (if there is a "rest") with '~'.
    while visual_y < view_vb.height {
        view_vb.draw_grapheme(0, visual_y, View::EMPTY_LINE_NOTATION, 0, reset_style());
        visual_y += 1;
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
    if !view.has_status_bar {
        return;
    }

    if width < 1 {
        return;
    }

    let mut status_bar_vb = VisualBox::new(width, 1);

    let mut status_bar_text = String::new();

    match &view.file_path {
        Some(path) => {
            // Add a * before the filename if it's been edited and not saved.
            if view.dirty {
                status_bar_text.push('*');
            }
            status_bar_text.push_str(path.to_str().unwrap())
        }
        None => {}
    }

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

    status_bar_vb.draw_with_style(0, 0, &status_bar_text, 0, status_bar_style);

    term.draw_visual_box(x, y, status_bar_vb);
}

fn render_command_bar(editor: &Editor, term: &mut Terminal, x: usize, y: usize, width: usize) {
    let mut command_input_vb = VisualBox::new(width, 1);

    // First, we draw the text entered by the user.
    let mut prompt_start = match &editor.mode {
        Mode::Command => String::from(">> "),
        Mode::Edit => String::from("#> "),
    };

    prompt_start.push_str(&editor.command_bar.view.buffer.text.to_string());
    command_input_vb.draw(0, 0, &prompt_start, 0);

    // Next, we draw the tab tracker. This is so the tab tracker renders in front of the prompt, should the prompt be too long.
    let mut tab_tracker = String::from("");

    for i in 0..editor.tabs.len() {
        if i == editor.focused_tab_idx {
            tab_tracker.push('[');
            tab_tracker.push_str(&(i + 1).to_string());
            tab_tracker.push(']');
        } else {
            tab_tracker.push_str(&(i + 1).to_string());
        }
    }

    let tab_tracker_x_coord = width - tab_tracker.len();
    command_input_vb.draw(tab_tracker_x_coord, 0, &tab_tracker, 0);

    term.draw_visual_box(x, y, command_input_vb);
}

fn get_visual_cursor_pos(
    editor: &Editor,
    ui_descriptor: &UiDescriptor,
    tab: &Tab,
    height: usize,
) -> (usize, usize) {
    match editor.mode {
        Mode::Edit => {
            for bound in &ui_descriptor.views {
                if bound.path == tab.focused_view_path {
                    let view = tab.get_current_focused_view();
                    let (abs_x, abs_y) = view.get_visual_cursor_pos();
                    return (
                        bound.x + abs_x - view.start_col,
                        bound.y + abs_y - view.start_row,
                    );
                }
            }

            unreachable!(
                "UiDescriptor::focused_path did not point to an actual path in the focused tab. File a bug report!"
            );
        }
        Mode::Command => {
            let view = &editor.command_bar.view;
            let (cursor_x, cursor_y) = view.get_visual_cursor_pos();
            return (
                cursor_x + 3 - view.start_col,
                cursor_y + height - 1 - view.start_row,
            );
        }
    }
}
