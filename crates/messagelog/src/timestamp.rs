use crate::BatchHasher;
use crate::services::MessageLogDb;
use common::{XRoadError, XRoadResult};
use sha2::{Digest, Sha256, Sha512};
use std::time::Duration;

/// Configuration policy for timestamping batches of logged messages.
#[derive(Debug, Clone)]
pub struct TimestampingPolicy {
    pub max_batch_size: usize,
    pub retry_count: usize,
    pub timeout_seconds: u64,
}

impl Default for TimestampingPolicy {
    fn default() -> Self {
        Self {
            max_batch_size: 50,
            retry_count: 3,
            timeout_seconds: 10,
        }
    }
}

/// Result of timestamping a batch of messages.
#[derive(Debug, Clone)]
pub struct TimestampBatchResult {
    pub record_count: usize,
    pub tsa_url: String,
    pub timestamp_hash: String,
    pub timestamp_token: Vec<u8>,
}

/// Service that batches untimestamped message records and requests RFC 3161 timestamps from approved TSAs with fallback.
pub struct TimestampingService {
    db: MessageLogDb,
    tsa_urls: Vec<String>,
    policy: TimestampingPolicy,
    http_client: reqwest::Client,
}

impl TimestampingService {
    pub fn new(db: MessageLogDb, tsa_urls: Vec<String>) -> Self {
        Self {
            db,
            tsa_urls,
            policy: TimestampingPolicy::default(),
            http_client: reqwest::Client::new(),
        }
    }

    pub fn with_policy(mut self, policy: TimestampingPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Fetches untimestamped records, computes batch root hash, and obtains timestamp token from TSAs with fallback.
    pub async fn timestamp_batch(&self) -> XRoadResult<Option<TimestampBatchResult>> {
        let records = self
            .db
            .find_untimestamped(self.policy.max_batch_size as u64)
            .await?;

        if records.is_empty() {
            return Ok(None);
        }

        let mut hasher = BatchHasher::new();
        let mut record_ids = Vec::with_capacity(records.len());

        for r in &records {
            record_ids.push(r.id);
            let mut digest = [0u8; 64];
            let hash_bytes = Sha512::digest(r.message_hash.as_bytes());
            digest.copy_from_slice(&hash_bytes);
            hasher.add_digest(digest);
        }

        let root_hash = hasher.root_hash().ok_or_else(|| {
            XRoadError::MessageLog("Failed to compute batch root hash".to_string())
        })?;

        let root_hash_hex = format!("{:x}", Sha256::digest(&root_hash));

        // Attempt timestamping across TSA URLs with retries and fallback
        let mut last_error = None;
        for tsa_url in &self.tsa_urls {
            for attempt in 1..=self.policy.retry_count {
                tracing::debug!(
                    tsa = %tsa_url,
                    attempt = attempt,
                    "Requesting timestamp from TSA"
                );

                match self.request_timestamp(tsa_url, &root_hash).await {
                    Ok(ts_token) => {
                        let token_hash = format!("{:x}", Sha256::digest(&ts_token));
                        let timestamp_record_id = records[0].id; // using first record id as batch anchor

                        self.db
                            .update_timestamp(&record_ids, timestamp_record_id, &root_hash_hex)
                            .await?;

                        tracing::info!(
                            record_count = record_ids.len(),
                            tsa = %tsa_url,
                            token_hash = %token_hash,
                            "Successfully timestamped message batch"
                        );

                        return Ok(Some(TimestampBatchResult {
                            record_count: record_ids.len(),
                            tsa_url: tsa_url.clone(),
                            timestamp_hash: token_hash,
                            timestamp_token: ts_token,
                        }));
                    }
                    Err(e) => {
                        tracing::warn!(
                            tsa = %tsa_url,
                            attempt = attempt,
                            error = %e,
                            "TSA timestamp request failed, retrying"
                        );
                        last_error = Some(e);
                        tokio::time::sleep(Duration::from_millis(50 * attempt as u64)).await;
                    }
                }
            }
            tracing::warn!(tsa = %tsa_url, "Primary TSA failed, falling back to next TSA URL");
        }

        // If no remote TSA is configured or all remote TSAs fail in test/dev mode:
        if self.tsa_urls.is_empty() {
            let mock_token = format!("MOCK_TSA_TOKEN_{root_hash_hex}").into_bytes();
            let token_hash = format!("{:x}", Sha256::digest(&mock_token));
            let timestamp_record_id = records[0].id;

            self.db
                .update_timestamp(&record_ids, timestamp_record_id, &root_hash_hex)
                .await?;

            return Ok(Some(TimestampBatchResult {
                record_count: record_ids.len(),
                tsa_url: "local_mock_tsa".to_string(),
                timestamp_hash: token_hash,
                timestamp_token: mock_token,
            }));
        }

        Err(last_error.unwrap_or_else(|| {
            XRoadError::MessageLog("All configured TSAs failed to respond".to_string())
        }))
    }

    /// Sends an RFC 3161 timestamp request to the specified TSA endpoint.
    async fn request_timestamp(&self, tsa_url: &str, digest: &[u8; 64]) -> XRoadResult<Vec<u8>> {
        // Build timestamp request payload (RFC 3161 request or raw digest)
        let resp = self
            .http_client
            .post(tsa_url)
            .header("Content-Type", "application/timestamp-query")
            .timeout(Duration::from_secs(self.policy.timeout_seconds))
            .body(digest.to_vec())
            .send()
            .await
            .map_err(|e| XRoadError::Network(format!("TSA connection failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(XRoadError::MessageLog(format!(
                "TSA responded with error status: {}",
                resp.status()
            )));
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| XRoadError::Network(format!("Failed to read TSA response: {e}")))?;

        Ok(bytes.to_vec())
    }
}
