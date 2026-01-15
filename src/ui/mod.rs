pub mod visual_box;

use crossterm::style::Attributes;
use crossterm::style::Color;
use crossterm::style::ContentStyle;

use crate::editor::Mode;
use crate::ui::visual_box::VisualBox;
use crate::{editor::Editor, terminal::Terminal};

use crate::editor::tab::{Axis, Layout};
use crate::editor::view::View;

/// Represents the bounding box and path for a single view
pub struct ViewBounds {
    pub path: Vec<bool>,
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

/// Contains all information needed to render a tab's layout
pub struct UiDescriptor {
    pub views: Vec<ViewBounds>,
    pub focused_path: Vec<bool>,
}

impl UiDescriptor {
    /// Constructs a UiDescriptor from a tab and terminal dimensions
    pub fn from_tab(tab: &crate::editor::tab::Tab, width: usize, height: usize) -> Self {
        let mut views = Vec::new();
        let focused_path = tab.get_focused_view_path().clone();
        
        Self::collect_view_bounds(&tab.layout, &mut views, Vec::new(), 0, 0, width, height);
        
        Self {
            views,
            focused_path,
        }
    }
    
    /// Recursively collects all view bounds from the layout tree
    fn collect_view_bounds(
        layout: &Layout,
        views: &mut Vec<ViewBounds>,
        current_path: Vec<bool>,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) {
        match layout {
            Layout::Leaf(_) => {
                views.push(ViewBounds {
                    path: current_path,
                    x,
                    y,
                    width,
                    height,
                });
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
                    
                    let mut left_path = current_path.clone();
                    left_path.push(false);
                    Self::collect_view_bounds(&children[0], views, left_path, x, y, left_width, height);
                    
                    let mut right_path = current_path.clone();
                    right_path.push(true);
                    Self::collect_view_bounds(&children[1], views, right_path, right_x, y, right_width, height);
                }
                Axis::Horizontal => {
                    let upper_height = (height as f32 * weight) as usize;
                    let mut lower_height = (height as f32 - (height as f32 * weight)) as usize;
                    
                    // Avoid off-by-one errors when applying split weights.
                    if upper_height + lower_height != height {
                        lower_height += 1;
                    }
                    let lower_y = y + upper_height;
                    
                    let mut upper_path = current_path.clone();
                    upper_path.push(false);
                    Self::collect_view_bounds(&children[0], views, upper_path, x, y, width, upper_height);
                    
                    let mut lower_path = current_path.clone();
                    lower_path.push(true);
                    Self::collect_view_bounds(&children[1], views, lower_path, x, lower_y, width, lower_height);
                }
            },
        }
    }
}

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
        let view_layout = tab.get_layout_at(&view_bounds.path)
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

    render_view_status_bar(view, term, bounds.x, bounds.y + bounds.height - 1, bounds.width, is_focused);

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