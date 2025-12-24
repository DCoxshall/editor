use crate::editor::view::View;

/// Is the given split a horizontal or vertical split?
pub enum Axis {
    Horizontal,
    Vertical,
}

/// Represents the layout of the views within a single tab.
pub enum Layout {
    Leaf {
        view: View,
    },
    Split {
        axis: Axis,

        // A number between 0 and 1, representing how much space the left or upper child Layout
        // should take. A value of 0 means it should take up none of the available space, and a
        // value of 1 means it should take up all of the available space.
        weight: f32,
        children: [Box<Layout>; 2],
    },
}

/// A tab contains one or more views, representing a logical collection of views. The user can swap
/// between tabs like browser tabs, however, unlike a browser tab, an editor tab can contain more
/// than one view.
pub struct Tab {
    pub layout: Layout,

    /// Recursive path into the Layout. An empty path means the focused view is the top-level leaf.
    /// Otherwise, the first element indicates which child contains the focused view: true for the
    /// upper/left split, false for the lower/right split.
    focused_view_path: Vec<bool>,
}

impl Tab {
    /// Creates a new Tab from a Vector of Views.
    /// # Arguments
    /// * `views: Vec<View>`: views to be opened. If `views` is empty, we create a new empty view.
    pub fn new() -> Self {
        Self {
            layout: Layout::Leaf { view: View::new() },

            focused_view_path: vec![],
        }
    }

    pub fn from_view(view: View) -> Self {
        Self {
            layout: Layout::Leaf { view },
            focused_view_path: vec![],
        }
    }

    pub fn insert_new_view(&mut self, new_view: View, direction: Axis) {
        let focused_layout = self.get_current_layout_mut();

        let old_view = match focused_layout {
            Layout::Leaf { view } => std::mem::replace(view, View::new()),
            _ => unreachable!(),
        };

        *focused_layout = Layout::Split {
            axis: direction,
            weight: 0.5,
            children: [
                Box::new(Layout::Leaf { view: old_view }),
                Box::new(Layout::Leaf { view: new_view }),
            ],
        };

        // Optional: focus the newly created view
        self.focused_view_path.push(false);
    }

    fn get_current_layout_mut(&mut self) -> &mut Layout {
        let mut cur_layout = &mut self.layout;
        for dir in &self.focused_view_path {
            match cur_layout {
                Layout::Split { children, .. } => {
                    cur_layout = if *dir {
                        &mut children[0]
                    } else {
                        &mut children[1]
                    };
                }
                Layout::Leaf { .. } => unreachable!(),
            }
        }
        cur_layout
    }
}
