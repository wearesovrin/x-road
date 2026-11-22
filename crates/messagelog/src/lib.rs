pub mod archiver;
pub mod db;
pub mod entities;
pub mod migration;

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha512};

pub use archiver::{ArchiveInfo, ArchiveVerificationResult, MessageLogArchiver, verify_archive};
pub use db::{MessageLogDb, NewLogRecord};
pub use migration::Migrator;

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

    #[tokio::test]
    async fn test_message_log_db_in_memory() {
        let db = MessageLogDb::connect_in_memory()
            .await
            .expect("Failed to connect to in-memory messagelog db");

        let log_id = db
            .log_message(NewLogRecord {
                query_id: "query-abc-123".to_string(),
                x_request_id: Some("req-xyz-456".to_string()),
                client_id_str: Some("DEV/COM/12345/CLIENT".to_string()),
                service_id_str: Some("DEV/COM/12345/SERVICE".to_string()),
                discriminator: "REST".to_string(),
                message_hash: "abcd1234hash".to_string(),
                signature_hash: Some("sighash123".to_string()),
                message_body: Some(r#"{"hello": "world"}"#.to_string()),
                signature_data: Some(vec![1, 2, 3, 4]),
                response: false,
            })
            .await
            .unwrap();

        assert!(log_id > 0);

        // Fetch by query id
        let record = db.get_by_query_id("query-abc-123").await.unwrap().unwrap();
        assert_eq!(record.id, log_id);
        assert_eq!(record.query_id, "query-abc-123");
        assert!(!record.archived);

        // Add attachment
        let att_id = db
            .add_attachment(log_id, 1, "atthash123", Some(b"dummy payload"))
            .await
            .unwrap();
        assert!(att_id > 0);

        let attachments = db.get_attachments(log_id).await.unwrap();
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].attachment_hash, "atthash123");

        // Update hash chain
        db.update_hash_chain(log_id, "chain_data_xyz", "root_hash_xyz")
            .await
            .unwrap();
        let updated = db.get_by_query_id("query-abc-123").await.unwrap().unwrap();
        assert_eq!(updated.hash_chain, Some("chain_data_xyz".to_string()));
        assert_eq!(updated.hash_chain_result, Some("root_hash_xyz".to_string()));

        // Check unarchived records and archive
        let unarchived = db.find_unarchived(10).await.unwrap();
        assert_eq!(unarchived.len(), 1);

        db.mark_archived(&[log_id]).await.unwrap();
        let unarchived_after = db.find_unarchived(10).await.unwrap();
        assert!(unarchived_after.is_empty());
    }
}
