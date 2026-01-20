/// Represents the bounding box and path for a single view. Belongs under the UI module because it's
/// used for determining the size of a view on screen.
pub struct ViewBounds {
    pub path: Vec<bool>,
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl ViewBounds {
    pub fn contains(&self, target_x: usize, target_y: usize) -> bool {
        if target_x < self.x
            || target_x >= self.x + self.width
            || target_y < self.y
            || target_y >= self.y + self.height
        {
            return false;
        }

        true
    }
}
