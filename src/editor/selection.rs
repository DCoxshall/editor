#[derive(Clone, PartialEq)]
pub struct Selection {
    /// Where the selection started from.
    pub anchor: usize,

    /// Represents whether the selection is actually in usage.
    pub active: bool,
}

impl Selection {
    pub fn new() -> Selection {
        Selection { anchor: 0, active: false }
    }

    pub fn update(&mut self, new_val: usize) {
        
    }
}