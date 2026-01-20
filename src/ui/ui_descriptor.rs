use crate::editor::tab::Axis;
use crate::editor::tab::Layout;
use super::ViewBounds;

/// Contains all information needed to render a tab's layout.
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
                    Self::collect_view_bounds(
                        &children[0],
                        views,
                        left_path,
                        x,
                        y,
                        left_width,
                        height,
                    );

                    let mut right_path = current_path.clone();
                    right_path.push(true);
                    Self::collect_view_bounds(
                        &children[1],
                        views,
                        right_path,
                        right_x,
                        y,
                        right_width,
                        height,
                    );
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
                    Self::collect_view_bounds(
                        &children[0],
                        views,
                        upper_path,
                        x,
                        y,
                        width,
                        upper_height,
                    );

                    let mut lower_path = current_path.clone();
                    lower_path.push(true);
                    Self::collect_view_bounds(
                        &children[1],
                        views,
                        lower_path,
                        x,
                        lower_y,
                        width,
                        lower_height,
                    );
                }
            },
        }
    }
}
