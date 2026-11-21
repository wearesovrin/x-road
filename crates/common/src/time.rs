use std::time::{Duration, Instant};

/// Call budget tracker for bounds on execution and timeouts across X-Road proxies.
#[derive(Debug, Clone)]
pub struct CallBudget {
    start: Instant,
    budget: Duration,
}

impl CallBudget {
    pub fn new(budget: Duration) -> Self {
        Self {
            start: Instant::now(),
            budget,
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn remaining(&self) -> Duration {
        self.budget.saturating_sub(self.elapsed())
    }

    pub fn is_expired(&self) -> bool {
        self.elapsed() >= self.budget
    }
}
