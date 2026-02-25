use std::time::Instant;

pub struct FastClock {
    base_instant: Instant,
    base_system: Instant,
}

impl FastClock {
    pub fn new() -> Self {
        Self {
            base_instant: Instant::now(),
            base_system: Instant::now()
        }
    }

	pub fn now(&self) -> Instant {
		self.base_system + self.base_instant.elapsed()
	}
}
