use super::ViewBounds;
use crate::editor::tab::{Axis, Layout, Tab};

/// Contains all information needed to render a tab's layout. A UiDescriptor can be created from a
/// Tab, with the new UiDescriptor containing a collection of ViewBounds corresponding to each View
/// in the tab.
pub struct UiDescriptor {
    pub views: Vec<ViewBounds>,
    pub focused_path: Vec<bool>,
}

impl UiDescriptor {
    pub fn from_tab(tab: &Tab, width: usize, height: usize) -> Self {
        let mut views = Vec::new();
        Self::generate_view_bounds(&tab.layout, &mut views, Vec::new(), 0, 0, width, height);
        Self {
            views,
            focused_path: tab.get_focused_view_path().clone(),
        }
    }

    /// Recursively generate the ViewBounds for each view in the layout, appending each one to `views`.
    fn generate_view_bounds(
        layout: &Layout,
        views: &mut Vec<ViewBounds>,
        path: Vec<bool>,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
    ) {
        match layout {
            Layout::Leaf(_) => views.push(ViewBounds {
                path,
                x,
                y,
                width: w,
                height: h,
            }),
            Layout::Split {
                axis,
                weight,
                children,
            } => match axis {
                Axis::Vertical => Self::split_vertical(children, weight, views, path, x, y, w, h),
                Axis::Horizontal => {
                    Self::split_horizontal(children, weight, views, path, x, y, w, h)
                }
            },
        }
    }

    fn split_vertical(
        children: &[Box<Layout>; 2],
        weight: &f32,
        views: &mut Vec<ViewBounds>,
        path: Vec<bool>,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
    ) {
        let (left_w, right_w) = Self::calc_split_dims(w, *weight);

        Self::generate_view_bounds(
            &children[0],
            views,
            Self::extend_path(&path, false),
            x,
            y,
            left_w,
            h,
        );
        Self::generate_view_bounds(
            &children[1],
            views,
            Self::extend_path(&path, true),
            x + left_w,
            y,
            right_w,
            h,
        );
    }

    fn split_horizontal(
        children: &[Box<Layout>; 2],
        weight: &f32,
        views: &mut Vec<ViewBounds>,
        path: Vec<bool>,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
    ) {
        let (top_h, bot_h) = Self::calc_split_dims(h, *weight);

        Self::generate_view_bounds(
            &children[0],
            views,
            Self::extend_path(&path, false),
            x,
            y,
            w,
            top_h,
        );
        Self::generate_view_bounds(
            &children[1],
            views,
            Self::extend_path(&path, true),
            x,
            y + top_h,
            w,
            bot_h,
        );
    }

    /// Calculates the primary and secondary sizes for a split, adjusting for rounding.
    fn calc_split_dims(total: usize, weight: f32) -> (usize, usize) {
        let first = (total as f32 * weight) as usize;
        let second = total - first; // This naturally handles the off-by-one correction
        (first, second)
    }

    fn extend_path(path: &Vec<bool>, branch: bool) -> Vec<bool> {
        let mut new_path = path.clone();
        new_path.push(branch);
        new_path
    }
}
