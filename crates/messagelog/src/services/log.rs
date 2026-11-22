use crate::entities::log_record;
use common::{MessageProtocol, XRoadError, XRoadResult};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};

/// New message record for insertion into the message log database.
#[derive(Debug, Clone)]
pub struct NewLogRecord {
    pub query_id: String,
    pub x_request_id: Option<String>,
    pub client_id_str: Option<String>,
    pub service_id_str: Option<String>,
    pub discriminator: MessageProtocol,
    pub message_hash: String,
    pub signature_hash: Option<String>,
    pub message_body: Option<String>,
    pub signature_data: Option<Vec<u8>>,
    pub response: bool,
}

/// Service for managing log records in Message Log.
#[derive(Clone, Copy, Default, Debug)]
pub struct LogService;

impl LogService {
    /// Inserts a new logged message record.
    pub async fn log_message(db: &DatabaseConnection, record: NewLogRecord) -> XRoadResult<i64> {
        let model = log_record::ActiveModel {
            query_id: Set(record.query_id),
            x_request_id: Set(record.x_request_id),
            client_id_str: Set(record.client_id_str),
            service_id_str: Set(record.service_id_str),
            discriminator: Set(record.discriminator.to_string()),
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
        .insert(db)
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to insert log record: {e}")))?;

        Ok(model.id)
    }

    /// Finds log record by query ID.
    pub async fn get_by_query_id(
        db: &DatabaseConnection,
        query_id: &str,
    ) -> XRoadResult<Option<log_record::Model>> {
        log_record::Entity::find()
            .filter(log_record::Column::QueryId.eq(query_id))
            .one(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query log record: {e}")))
    }

    /// Finds unarchived log records up to `limit`.
    pub async fn find_unarchived(
        db: &DatabaseConnection,
        limit: u64,
    ) -> XRoadResult<Vec<log_record::Model>> {
        log_record::Entity::find()
            .filter(log_record::Column::Archived.eq(false))
            .order_by_asc(log_record::Column::Id)
            .limit(limit)
            .all(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query unarchived records: {e}")))
    }

    /// Marks the given log records as archived.
    pub async fn mark_archived(db: &DatabaseConnection, ids: &[i64]) -> XRoadResult<()> {
        log_record::Entity::update_many()
            .col_expr(
                log_record::Column::Archived,
                sea_orm::sea_query::Expr::val(true),
            )
            .filter(log_record::Column::Id.is_in(ids.to_vec()))
            .exec(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to mark records archived: {e}")))?;

        Ok(())
    }

    /// Finds message records that have not yet been timestamped.
    pub async fn find_untimestamped(
        db: &DatabaseConnection,
        limit: u64,
    ) -> XRoadResult<Vec<log_record::Model>> {
        log_record::Entity::find()
            .filter(log_record::Column::TimestampRecordId.is_null())
            .order_by_asc(log_record::Column::Id)
            .limit(limit)
            .all(db)
            .await
            .map_err(|e| {
                XRoadError::Database(format!("Failed to query untimestamped records: {e}"))
            })
    }
}
