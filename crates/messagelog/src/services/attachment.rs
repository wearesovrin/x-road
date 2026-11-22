use crate::entities::attachment;
use common::{XRoadError, XRoadResult};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use std::sync::Arc;

/// Service for managing attachments in Message Log.
#[derive(Clone)]
pub struct AttachmentService {
    db: Arc<DatabaseConnection>,
}

impl AttachmentService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn raw_connection(&self) -> &DatabaseConnection {
        &self.db
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
}
