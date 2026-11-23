use crate::error::{XRoadError, XRoadResult};
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

    pub fn check_expired(&self) -> XRoadResult<()> {
        if self.is_expired() {
            Err(XRoadError::Timeout)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_budget() {
        let budget = CallBudget::new(Duration::from_millis(50));
        assert!(!budget.is_expired());
        assert!(budget.check_expired().is_ok());
        assert!(budget.remaining() > Duration::ZERO);

        let expired_budget = CallBudget {
            start: Instant::now() - Duration::from_secs(10),
            budget: Duration::from_secs(1),
        };
        assert!(expired_budget.is_expired());
        assert_eq!(expired_budget.remaining(), Duration::ZERO);
        assert!(matches!(
            expired_budget.check_expired(),
            Err(XRoadError::Timeout)
        ));
    }
}
