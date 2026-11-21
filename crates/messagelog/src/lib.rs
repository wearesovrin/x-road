use chrono::{DateTime, Utc};
use sha2::{Digest, Sha512};

/// Logged message record stored for non-repudiation and auditing.
#[derive(Debug, Clone)]
pub struct MessageRecord {
    pub id: i64,
    pub query_id: String,
    pub message_hash: Vec<u8>,
    pub signature_hash: Vec<u8>,
    pub timestamp: DateTime<Utc>,
    pub signature: Vec<u8>,
}

/// Freudenthal batch hasher for building verifiable hash chains of messages.
pub struct BatchHasher {
    digests: Vec<[u8; 64]>,
}

impl BatchHasher {
    pub fn new() -> Self {
        Self {
            digests: Vec::new(),
        }
    }

    pub fn add_digest(&mut self, digest: [u8; 64]) {
        self.digests.push(digest);
    }

    /// Computes root hash of the batch tree.
    pub fn root_hash(&self) -> Option<[u8; 64]> {
        if self.digests.is_empty() {
            return None;
        }

        let mut current_level = self.digests.clone();
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in current_level.chunks(2) {
                let mut hasher = Sha512::new();
                hasher.update(&chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(&chunk[1]);
                } else {
                    hasher.update(&chunk[0]);
                }
                next_level.push(hasher.finalize().into());
            }
            current_level = next_level;
        }

        Some(current_level[0])
    }
}

impl Default for BatchHasher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_hasher() {
        let mut hasher = BatchHasher::new();
        hasher.add_digest([1u8; 64]);
        hasher.add_digest([2u8; 64]);
        let root = hasher.root_hash();
        assert!(root.is_some());
    }
}
