use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Thread-safe in-memory storage for pending ACME HTTP-01 challenges.
///
/// When an ACME server issues an HTTP-01 challenge, the token and key authorization
/// are registered here and served at `/.well-known/acme-challenge/<token>`.
#[derive(Debug, Clone, Default)]
pub struct AcmeChallengeStore {
    challenges: Arc<RwLock<HashMap<String, String>>>,
}

impl AcmeChallengeStore {
    pub fn new() -> Self {
        Self {
            challenges: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Stores a key authorization response for a given challenge token.
    pub fn insert(&self, token: impl Into<String>, key_authorization: impl Into<String>) {
        if let Ok(mut map) = self.challenges.write() {
            map.insert(token.into(), key_authorization.into());
        }
    }

    /// Retrieves the key authorization for a given challenge token.
    pub fn get(&self, token: &str) -> Option<String> {
        self.challenges.read().ok()?.get(token).cloned()
    }

    /// Removes a challenge token once validation is complete.
    pub fn remove(&self, token: &str) -> Option<String> {
        self.challenges.write().ok()?.remove(token)
    }

    /// Clears all challenges.
    pub fn clear(&self) {
        if let Ok(mut map) = self.challenges.write() {
            map.clear();
        }
    }
}
