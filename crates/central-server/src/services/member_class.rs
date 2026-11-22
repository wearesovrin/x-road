use crate::entities::member_class;
use common::{XRoadError, XRoadResult};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use std::sync::Arc;

#[derive(Clone)]
pub struct MemberClassService {
    db: Arc<DatabaseConnection>,
}

impl MemberClassService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub fn conn(&self) -> &DatabaseConnection {
        &self.db
    }

    pub async fn add_member_class(
        &self,
        code: &str,
        description: &str,
    ) -> XRoadResult<member_class::Model> {
        member_class::ActiveModel {
            code: Set(code.to_string()),
            description: Set(description.to_string()),
        }
        .insert(self.conn())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to add member class: {e}")))
    }

    pub async fn list_member_classes(&self) -> XRoadResult<Vec<member_class::Model>> {
        member_class::Entity::find()
            .all(self.conn())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list member classes: {e}")))
    }

    pub async fn get_member_class(&self, code: &str) -> XRoadResult<Option<member_class::Model>> {
        member_class::Entity::find_by_id(code.to_string())
            .one(self.conn())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get member class: {e}")))
    }

    pub async fn delete_member_class(&self, code: &str) -> XRoadResult<bool> {
        let res = member_class::Entity::delete_by_id(code.to_string())
            .exec(self.conn())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete member class: {e}")))?;
        Ok(res.rows_affected > 0)
    }
}
