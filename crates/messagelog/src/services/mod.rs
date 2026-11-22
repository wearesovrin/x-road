pub mod attachment;
pub mod hash_chain;
pub mod log;

pub use attachment::AttachmentService;
pub use hash_chain::HashChainService;
pub use log::{LogService, NewLogRecord};

use common::{XRoadError, XRoadResult};
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use std::sync::Arc;

/// SeaORM-backed Message Log repository and services.
#[derive(Clone)]
pub struct MessageLog {
    pub raw_db: Arc<DatabaseConnection>,
    pub logs: LogService,
    pub attachments: AttachmentService,
    pub hash_chains: HashChainService,
}

impl MessageLog {
    pub fn new(db: DatabaseConnection) -> Self {
        let db_arc = Arc::new(db);
        Self {
            logs: LogService::new(db_arc.clone()),
            attachments: AttachmentService::new(db_arc.clone()),
            hash_chains: HashChainService::new(db_arc.clone()),
            raw_db: db_arc,
        }
    }

    pub async fn connect(db_url: &str) -> XRoadResult<Self> {
        let db = Database::connect(db_url)
            .await
            .map_err(|e| XRoadError::Database(format!("Database connection failed: {e}")))?;
        crate::migration::Migrator::up(&db, None)
            .await
            .map_err(|e| XRoadError::Database(format!("Database migration failed: {e}")))?;
        Ok(Self::new(db))
    }

    pub async fn connect_in_memory() -> XRoadResult<Self> {
        Self::connect("sqlite::memory:").await
    }

    pub fn raw_connection(&self) -> &DatabaseConnection {
        &self.raw_db
    }

    // Direct access helper methods
    pub async fn log_message(&self, record: NewLogRecord) -> XRoadResult<i64> {
        self.logs.log_message(record).await
    }

    pub async fn get_by_query_id(
        &self,
        query_id: &str,
    ) -> XRoadResult<Option<crate::entities::log_record::Model>> {
        self.logs.get_by_query_id(query_id).await
    }

    pub async fn find_unarchived(
        &self,
        limit: u64,
    ) -> XRoadResult<Vec<crate::entities::log_record::Model>> {
        self.logs.find_unarchived(limit).await
    }

    pub async fn mark_archived(&self, ids: &[i64]) -> XRoadResult<()> {
        self.logs.mark_archived(ids).await
    }

    pub async fn find_untimestamped(
        &self,
        limit: u64,
    ) -> XRoadResult<Vec<crate::entities::log_record::Model>> {
        self.logs.find_untimestamped(limit).await
    }

    pub async fn add_attachment(
        &self,
        logrecord_id: i64,
        attachment_no: i32,
        hash: &str,
        data: Option<&[u8]>,
    ) -> XRoadResult<i64> {
        self.attachments
            .add_attachment(logrecord_id, attachment_no, hash, data)
            .await
    }

    pub async fn get_attachments(
        &self,
        logrecord_id: i64,
    ) -> XRoadResult<Vec<crate::entities::attachment::Model>> {
        self.attachments.get_attachments(logrecord_id).await
    }

    pub async fn update_hash_chain(
        &self,
        logrecord_id: i64,
        hash_chain: &str,
        hash_chain_result: &str,
    ) -> XRoadResult<()> {
        self.hash_chains
            .update_hash_chain(logrecord_id, hash_chain, hash_chain_result)
            .await
    }

    pub async fn update_timestamp(
        &self,
        record_ids: &[i64],
        timestamp_record_id: i64,
        timestamp_hash_chain: &str,
    ) -> XRoadResult<()> {
        self.hash_chains
            .update_timestamp(record_ids, timestamp_record_id, timestamp_hash_chain)
            .await
    }
}

pub type MessageLogDb = MessageLog;
