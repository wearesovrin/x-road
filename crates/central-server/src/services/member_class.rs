use crate::entities::member_class;
use common::{XRoadError, XRoadResult};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

/// Service for managing Member Classes in Central Server.
#[derive(Clone, Copy, Default, Debug)]
pub struct MemberClassService;

impl MemberClassService {
    pub async fn add_member_class(
        db: &DatabaseConnection,
        code: &str,
        description: &str,
    ) -> XRoadResult<member_class::Model> {
        member_class::ActiveModel {
            code: Set(code.to_string()),
            description: Set(description.to_string()),
        }
        .insert(db)
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to add member class: {e}")))
    }

    pub async fn list_member_classes(
        db: &DatabaseConnection,
    ) -> XRoadResult<Vec<member_class::Model>> {
        member_class::Entity::find()
            .all(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list member classes: {e}")))
    }

    pub async fn get_member_class(
        db: &DatabaseConnection,
        code: &str,
    ) -> XRoadResult<Option<member_class::Model>> {
        member_class::Entity::find_by_id(code.to_string())
            .one(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get member class: {e}")))
    }

    pub async fn delete_member_class(db: &DatabaseConnection, code: &str) -> XRoadResult<bool> {
        let res = member_class::Entity::delete_by_id(code.to_string())
            .exec(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete member class: {e}")))?;
        Ok(res.rows_affected > 0)
    }
}
