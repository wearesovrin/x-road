use crate::entities::{attachment, log_record};
use crate::migration::Migrator;
use common::{XRoadError, XRoadResult};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use sea_orm_migration::MigratorTrait;
use std::sync::Arc;

/// New message record for insertion into the message log database.
#[derive(Debug, Clone)]
pub struct NewLogRecord {
    pub query_id: String,
    pub x_request_id: Option<String>,
    pub client_id_str: Option<String>,
    pub service_id_str: Option<String>,
    pub discriminator: String,
    pub message_hash: String,
    pub signature_hash: Option<String>,
    pub message_body: Option<String>,
    pub signature_data: Option<Vec<u8>>,
    pub response: bool,
}

/// SeaORM-backed Message Log database repository.
#[derive(Clone)]
pub struct MessageLogDb {
    db: Arc<DatabaseConnection>,
}

impl MessageLogDb {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db: Arc::new(db) }
    }

    /// Connects to database at `db_url` and runs pending migrations.
    pub async fn connect(db_url: &str) -> XRoadResult<Self> {
        let db = Database::connect(db_url)
            .await
            .map_err(|e| XRoadError::Database(format!("Database connection failed: {e}")))?;
        Migrator::up(&db, None)
            .await
            .map_err(|e| XRoadError::Database(format!("Database migration failed: {e}")))?;
        Ok(Self::new(db))
    }

    /// Connects to in-memory SQLite database and runs migrations.
    pub async fn connect_in_memory() -> XRoadResult<Self> {
        Self::connect("sqlite::memory:").await
    }

    pub fn raw_connection(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Inserts a new logged message record.
    pub async fn log_message(&self, record: NewLogRecord) -> XRoadResult<i64> {
        let model = log_record::ActiveModel {
            query_id: Set(record.query_id),
            x_request_id: Set(record.x_request_id),
            client_id_str: Set(record.client_id_str),
            service_id_str: Set(record.service_id_str),
            discriminator: Set(record.discriminator),
            message_hash: Set(record.message_hash),
            signature_hash: Set(record.signature_hash),
            message_body: Set(record.message_body),
            signature_data: Set(record.signature_data),
            hash_chain: Set(None),
            hash_chain_result: Set(None),
            timestamp_record_id: Set(None),
            timestamp_hash_chain: Set(None),
            response: Set(record.response),
            archived: Set(false),
            time: Set(chrono::Utc::now()),
            ..Default::default()
        }
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to insert log record: {e}")))?;

        Ok(model.id)
    }

    /// Finds log record by query ID.
    pub async fn get_by_query_id(&self, query_id: &str) -> XRoadResult<Option<log_record::Model>> {
        log_record::Entity::find()
            .filter(log_record::Column::QueryId.eq(query_id))
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query log record: {e}")))
    }

    /// Finds unarchived log records up to `limit`.
    pub async fn find_unarchived(&self, limit: u64) -> XRoadResult<Vec<log_record::Model>> {
        log_record::Entity::find()
            .filter(log_record::Column::Archived.eq(false))
            .order_by_asc(log_record::Column::Id)
            .limit(limit)
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query unarchived records: {e}")))
    }

    /// Marks the given log records as archived.
    pub async fn mark_archived(&self, ids: &[i64]) -> XRoadResult<()> {
        log_record::Entity::update_many()
            .col_expr(
                log_record::Column::Archived,
                sea_orm::sea_query::Expr::val(true),
            )
            .filter(log_record::Column::Id.is_in(ids.to_vec()))
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to mark records archived: {e}")))?;

        Ok(())
    }

    /// Stores an attachment associated with a log record.
    pub async fn add_attachment(
        &self,
        logrecord_id: i64,
        attachment_no: i32,
        hash: &str,
        data: Option<&[u8]>,
    ) -> XRoadResult<i64> {
        let model = attachment::ActiveModel {
            logrecord_id: Set(logrecord_id),
            attachment_no: Set(attachment_no),
            attachment_hash: Set(hash.to_string()),
            data: Set(data.map(|d| d.to_vec())),
            ..Default::default()
        }
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to insert attachment: {e}")))?;

        Ok(model.id)
    }

    /// Retrieves all attachments for a log record.
    pub async fn get_attachments(&self, logrecord_id: i64) -> XRoadResult<Vec<attachment::Model>> {
        attachment::Entity::find()
            .filter(attachment::Column::LogrecordId.eq(logrecord_id))
            .order_by_asc(attachment::Column::AttachmentNo)
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query attachments: {e}")))
    }

    /// Updates the Freudenthal hash chain and root hash result for a log record.
    pub async fn update_hash_chain(
        &self,
        logrecord_id: i64,
        hash_chain: &str,
        hash_chain_result: &str,
    ) -> XRoadResult<()> {
        log_record::Entity::update_many()
            .col_expr(
                log_record::Column::HashChain,
                sea_orm::sea_query::Expr::val(hash_chain),
            )
            .col_expr(
                log_record::Column::HashChainResult,
                sea_orm::sea_query::Expr::val(hash_chain_result),
            )
            .filter(log_record::Column::Id.eq(logrecord_id))
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to update hash chain: {e}")))?;

        Ok(())
    }

    /// Finds message records that have not yet been timestamped.
    pub async fn find_untimestamped(&self, limit: u64) -> XRoadResult<Vec<log_record::Model>> {
        log_record::Entity::find()
            .filter(log_record::Column::TimestampRecordId.is_null())
            .order_by_asc(log_record::Column::Id)
            .limit(limit)
            .all(self.raw_connection())
            .await
            .map_err(|e| {
                XRoadError::Database(format!("Failed to query untimestamped records: {e}"))
            })
    }

    /// Updates records with the timestamp token record ID and the timestamp hash chain.
    pub async fn update_timestamp(
        &self,
        record_ids: &[i64],
        timestamp_record_id: i64,
        timestamp_hash_chain: &str,
    ) -> XRoadResult<()> {
        log_record::Entity::update_many()
            .col_expr(
                log_record::Column::TimestampRecordId,
                sea_orm::sea_query::Expr::val(timestamp_record_id),
            )
            .col_expr(
                log_record::Column::TimestampHashChain,
                sea_orm::sea_query::Expr::val(timestamp_hash_chain),
            )
            .filter(log_record::Column::Id.is_in(record_ids.to_vec()))
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to update timestamp record: {e}")))?;

        Ok(())
    }
}
