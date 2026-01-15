use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::editor::view::View;
use crate::ui::UiDescriptor;
use crate::terminal::Terminal;

/// Is the given split a horizontal or vertical split?
pub enum Axis {
    Horizontal,
    Vertical,
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
            match key_event.code {
                KeyCode::Up => {
                    let (width, height) = terminal.size();
                    self.move_focus_up(width, height);
                }
                KeyCode::Down => {
                    let (width, height) = terminal.size();
                    self.move_focus_down(width, height);
                }
                KeyCode::Left => {
                    let (width, height) = terminal.size();
                    self.move_focus_left(width, height);
                }
                KeyCode::Right => {
                    let (width, height) = terminal.size();
                    self.move_focus_right(width, height);
                }
                _ => {
                    // If Alt is held but it's not a focus movement key, pass to view
                    self.get_current_focused_view_mut().handle_keystroke(key_event);
                }
            }
        } else {
            // No Alt modifier, pass to view
            self.get_current_focused_view_mut().handle_keystroke(key_event);
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

    /// Moves focus to the view above the currently focused view.
    /// Returns true if focus was moved, false if no suitable view was found.
    pub fn move_focus_up(&mut self, width: usize, height: usize) -> bool {
        let descriptor = UiDescriptor::from_tab(self, width, height);
        self.move_focus_direction(&descriptor, |current, candidate| {
            // Check if candidate overlaps horizontally with current
            let horizontal_overlap = !(candidate.x + candidate.width <= current.x 
                || candidate.x >= current.x + current.width);
            
            // Check if candidate is above current
            let is_above = candidate.y + candidate.height <= current.y;
            
            if horizontal_overlap && is_above {
                Some((current.y - (candidate.y + candidate.height), 
                      (candidate.x as i32 - current.x as i32).abs()))
            } else {
                None
            }
        })
    }

    /// Moves focus to the view below the currently focused view.
    /// Returns true if focus was moved, false if no suitable view was found.
    pub fn move_focus_down(&mut self, width: usize, height: usize) -> bool {
        let descriptor = UiDescriptor::from_tab(self, width, height);
        self.move_focus_direction(&descriptor, |current, candidate| {
            // Check if candidate overlaps horizontally with current
            let horizontal_overlap = !(candidate.x + candidate.width <= current.x 
                || candidate.x >= current.x + current.width);
            
            // Check if candidate is below current
            let is_below = candidate.y >= current.y + current.height;
            
            if horizontal_overlap && is_below {
                Some((candidate.y - (current.y + current.height), 
                      (candidate.x as i32 - current.x as i32).abs()))
            } else {
                None
            }
        })
    }

    /// Moves focus to the view to the left of the currently focused view.
    /// Returns true if focus was moved, false if no suitable view was found.
    pub fn move_focus_left(&mut self, width: usize, height: usize) -> bool {
        let descriptor = UiDescriptor::from_tab(self, width, height);
        self.move_focus_direction(&descriptor, |current, candidate| {
            // Check if candidate overlaps vertically with current
            let vertical_overlap = !(candidate.y + candidate.height <= current.y 
                || candidate.y >= current.y + current.height);
            
            // Check if candidate is to the left of current
            let is_left = candidate.x + candidate.width <= current.x;
            
            if vertical_overlap && is_left {
                Some((current.x - (candidate.x + candidate.width), 
                      (candidate.y as i32 - current.y as i32).abs()))
            } else {
                None
            }
        })
    }

    /// Moves focus to the view to the right of the currently focused view.
    /// Returns true if focus was moved, false if no suitable view was found.
    pub fn move_focus_right(&mut self, width: usize, height: usize) -> bool {
        let descriptor = UiDescriptor::from_tab(self, width, height);
        self.move_focus_direction(&descriptor, |current, candidate| {
            // Check if candidate overlaps vertically with current
            let vertical_overlap = !(candidate.y + candidate.height <= current.y 
                || candidate.y >= current.y + current.height);
            
            // Check if candidate is to the right of current
            let is_right = candidate.x >= current.x + current.width;
            
            if vertical_overlap && is_right {
                Some((candidate.x - (current.x + current.width), 
                      (candidate.y as i32 - current.y as i32).abs()))
            } else {
                None
            }
        })
    }

    /// Helper function that finds the best candidate view in a given direction
    /// and updates the focused_view_path. The distance_fn should return Some((primary_distance, secondary_distance))
    /// if the candidate is valid, where primary_distance is the distance in the main direction
    /// and secondary_distance is used as a tiebreaker (typically horizontal distance for vertical moves,
    /// and vertical distance for horizontal moves).
    fn move_focus_direction<F>(&mut self, descriptor: &UiDescriptor, distance_fn: F) -> bool
    where
        F: Fn(&crate::ui::ViewBounds, &crate::ui::ViewBounds) -> Option<(usize, i32)>,
    {
        // Find the currently focused view
        let current_bounds = descriptor.views.iter()
            .find(|vb| vb.path == descriptor.focused_path)
            .expect("Focused view should exist in descriptor");

        // Find the best candidate
        let mut best_candidate: Option<(&crate::ui::ViewBounds, usize, i32)> = None;

        for candidate in &descriptor.views {
            if candidate.path == descriptor.focused_path {
                continue; // Skip the current view
            }

            if let Some((primary_dist, secondary_dist)) = distance_fn(current_bounds, candidate) {
                let is_better = match best_candidate {
                    None => true,
                    Some((_, best_primary, best_secondary)) => {
                        primary_dist < best_primary 
                        || (primary_dist == best_primary && secondary_dist < best_secondary)
                    }
                };

                if is_better {
                    best_candidate = Some((candidate, primary_dist, secondary_dist));
                }
            }
        }

        // Update focus if we found a candidate
        if let Some((candidate, _, _)) = best_candidate {
            self.focused_view_path = candidate.path.clone();
            true
        } else {
            false
        }
    }
}
