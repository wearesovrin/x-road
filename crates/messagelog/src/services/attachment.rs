use crate::entities::attachment;
use common::{XRoadError, XRoadResult};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};

/// Service for managing attachments in Message Log.
#[derive(Clone, Copy, Default, Debug)]
pub struct AttachmentService;

impl AttachmentService {
    /// Stores an attachment associated with a log record.
    pub async fn add_attachment(
        db: &DatabaseConnection,
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
        .insert(db)
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to insert attachment: {e}")))?;

        Ok(model.id)
    }

    /// Retrieves all attachments for a log record.
    pub async fn get_attachments(
        db: &DatabaseConnection,
        logrecord_id: i64,
    ) -> XRoadResult<Vec<attachment::Model>> {
        attachment::Entity::find()
            .filter(attachment::Column::LogrecordId.eq(logrecord_id))
            .order_by_asc(attachment::Column::AttachmentNo)
            .all(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query attachments: {e}")))
    }
}
