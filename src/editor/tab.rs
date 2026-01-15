use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::editor::view::View;
use crate::terminal::Terminal;
use crate::ui::UiDescriptor;

/// Is the given split a horizontal or vertical split?
pub enum Axis {
    Horizontal,
    Vertical,
}

/// Represents the four cardinal directions for focus movement
#[derive(PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Represents the layout of the views within a single tab.
pub enum Layout {
    Leaf(View),
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
    focused_view_path: Vec<bool>,
}

impl Tab {
    /// Creates a new Tab from a Vector of Views.
    /// # Arguments
    /// * `views: Vec<View>`: views to be opened. If `views` is empty, we create a new empty view.
    pub fn new() -> Self {
        Self {
            layout: Layout::Leaf(View::new()),
            focused_view_path: vec![],
        }
    }

    pub fn from_view(view: View) -> Self {
        Self {
            layout: Layout::Leaf(view),
            focused_view_path: vec![],
        }
    }

    pub fn handle_keystroke(&mut self, key_event: KeyEvent, terminal: &Terminal) {
        if key_event.modifiers.contains(KeyModifiers::ALT) {
            let (width, height) = terminal.size();
            match key_event.code {
                KeyCode::Up => {
                    self.move_focus_direction(Direction::Up, width, height);
                }
                KeyCode::Down => {
                    self.move_focus_direction(Direction::Down, width, height);
                }
                KeyCode::Left => {
                    self.move_focus_direction(Direction::Left, width, height);
                }
                KeyCode::Right => {
                    self.move_focus_direction(Direction::Right, width, height);
                }
                _ => {
                    // If Alt is held but it's not a focus movement key, pass to view
                    self.get_current_focused_view_mut()
                        .handle_keystroke(key_event);
                }
            }
        } else {
            // No Alt modifier, pass to view
            self.get_current_focused_view_mut()
                .handle_keystroke(key_event);
        }
    }

    pub fn insert_new_view(&mut self, new_view: View, direction: Axis) {
        let focused_leaf = self.get_focused_leaf_mut();

        let old_view = match focused_leaf {
            Layout::Leaf(view) => view.clone(),
            _ => unreachable!(),
        };

        *focused_leaf = Layout::Split {
            axis: direction,
            weight: 0.5,
            children: [
                Box::new(Layout::Leaf(old_view)),
                Box::new(Layout::Leaf(new_view)),
            ],
        };

        self.focused_view_path.push(false);
    }

    pub fn get_focused_view_path(&self) -> &Vec<bool> {
        &self.focused_view_path
    }

    /// If the path points past the edge of the tree, this method returns None. Otherwise, it
    /// returns a reference to the layout pointed to by path.
    pub fn get_layout_at(&self, path: &Vec<bool>) -> Option<&Layout> {
        let mut cur_layout = &self.layout;
        let mut consumed = 0;
        for dir in path {
            consumed += 1;
            match cur_layout {
                Layout::Split { children, .. } => {
                    cur_layout = if *dir { &children[1] } else { &children[0] }
                }
                Layout::Leaf(..) => return None,
            }
        }
        if consumed == path.len() {
            Some(cur_layout)
        } else {
            None
        }
    }

    /// If the path points past the edge of the tree, this method returns None. Otherwise, it
    /// returns a reference to the layout pointed to by path.
    fn get_layout_at_mut(&mut self, path: &Vec<bool>) -> Option<&mut Layout> {
        let mut cur_layout = &mut self.layout;
        let mut consumed = 0;
        for dir in path {
            consumed += 1;
            match cur_layout {
                Layout::Split { children, .. } => {
                    cur_layout = if *dir {
                        &mut children[1]
                    } else {
                        &mut children[0]
                    }
                }
                Layout::Leaf(..) => return None,
            }
        }
        if consumed == path.len() {
            Some(cur_layout)
        } else {
            None
        }
    }

    fn get_focused_leaf_mut(&mut self) -> &mut Layout {
        let focused_view_path = self.focused_view_path.clone();
        let focused_layout = self
            .get_layout_at_mut(&focused_view_path)
            .expect("Focused path pointed past the end of the focus tree. File a bug report!");
        match focused_layout {
            Layout::Split { .. } => {
                panic!("Focused path pointed to a split rather than a leaf. File a bug report!")
            }
            Layout::Leaf(..) => return focused_layout,
        }
    }

    pub fn get_current_focused_view_mut(&mut self) -> &mut View {
        let current_leaf = self.get_focused_leaf_mut();
        match current_leaf {
            Layout::Leaf(view) => view,
            Layout::Split { .. } => unreachable!(),
        }
    }

    /// Move the focus to the view which is in the corresponding direction of the current focused
    /// view.
    fn move_focus_direction(&mut self, direction: Direction, width: usize, height: usize) -> bool {
        let descriptor = UiDescriptor::from_tab(self, width, height);

        // Find the currently focused view's bounds.
        let cur_bounds = descriptor
            .views
            .iter()
            .find(|vb| vb.path == descriptor.focused_path)
            .expect("Focused view should exist in descriptor");

        // If we're trying to move off-screen, just return false.
        if (direction == Direction::Up && cur_bounds.y == 0)
            || (direction == Direction::Down && cur_bounds.y + cur_bounds.height == height)
            || (direction == Direction::Left && cur_bounds.x == 0)
            || (direction == Direction::Right && cur_bounds.x + cur_bounds.width == width)
        {
            return false;
        }

        let midpoint_x = (cur_bounds.x + cur_bounds.width) / 2;
        let midpoint_y = (cur_bounds.y + cur_bounds.height) / 2;

        let (target_x, target_y) = match direction {
            Direction::Up => (midpoint_x, cur_bounds.y - 1),
            Direction::Down => (midpoint_x, cur_bounds.y + cur_bounds.height),
            Direction::Left => (cur_bounds.x - 1, midpoint_y),
            Direction::Right => (cur_bounds.x + cur_bounds.width, midpoint_y),
        };

        for view_bound in descriptor.views {
            if view_bound.contains(target_x, target_y) {
                self.focused_view_path = view_bound.path;
                return true;
            }
        }

        false
    }
}
