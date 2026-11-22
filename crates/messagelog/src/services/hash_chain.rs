use crate::entities::log_record;
use common::{XRoadError, XRoadResult};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

/// Service for managing hash chains and timestamps in Message Log.
#[derive(Clone, Copy, Default, Debug)]
pub struct HashChainService;

impl HashChainService {
    /// Updates the Freudenthal hash chain and root hash result for a log record.
    pub async fn update_hash_chain(
        db: &DatabaseConnection,
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
            .exec(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to update hash chain: {e}")))?;

        Ok(())
    }

    /// Updates records with the timestamp token record ID and the timestamp hash chain.
    pub async fn update_timestamp(
        db: &DatabaseConnection,
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
            .exec(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to update timestamp record: {e}")))?;

        Ok(())
    }
}
