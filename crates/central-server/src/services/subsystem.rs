use crate::entities::subsystem;
use common::{XRoadError, XRoadResult};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

#[derive(Clone)]
pub struct SubsystemService {
    db: Arc<DatabaseConnection>,
}

impl SubsystemService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub fn conn(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Adds a subsystem to a member.
    pub async fn add_subsystem(
        &self,
        member_id: i64,
        subsystem_code: &str,
    ) -> XRoadResult<subsystem::Model> {
        subsystem::ActiveModel {
            member_id: Set(member_id),
            subsystem_code: Set(subsystem_code.to_string()),
            created_at: Set(chrono::Utc::now()),
            ..Default::default()
        }
        .insert(self.conn())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to insert subsystem: {e}")))
    }

    pub async fn get_subsystem(&self, id: i64) -> XRoadResult<Option<subsystem::Model>> {
        subsystem::Entity::find_by_id(id)
            .one(self.conn())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get subsystem: {e}")))
    }

    /// Lists all subsystems belonging to a member.
    pub async fn list_subsystems(&self, member_id: i64) -> XRoadResult<Vec<subsystem::Model>> {
        subsystem::Entity::find()
            .filter(subsystem::Column::MemberId.eq(member_id))
            .all(self.conn())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list subsystems: {e}")))
    }

    pub async fn list_all_subsystems(&self) -> XRoadResult<Vec<subsystem::Model>> {
        subsystem::Entity::find()
            .all(self.conn())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list all subsystems: {e}")))
    }

    pub async fn delete_subsystem(&self, id: i64) -> XRoadResult<bool> {
        let res = subsystem::Entity::delete_by_id(id)
            .exec(self.conn())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete subsystem: {e}")))?;
        Ok(res.rows_affected > 0)
    }
}
