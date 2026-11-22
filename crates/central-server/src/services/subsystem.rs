use crate::entities::subsystem;
use common::{XRoadError, XRoadResult};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

/// Service for managing Subsystems in Central Server.
#[derive(Clone, Copy, Default, Debug)]
pub struct SubsystemService;

impl SubsystemService {
    /// Adds a subsystem to a member.
    pub async fn add_subsystem(
        db: &DatabaseConnection,
        member_id: i64,
        subsystem_code: &str,
    ) -> XRoadResult<subsystem::Model> {
        subsystem::ActiveModel {
            member_id: Set(member_id),
            subsystem_code: Set(subsystem_code.to_string()),
            created_at: Set(chrono::Utc::now()),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to insert subsystem: {e}")))
    }

    pub async fn get_subsystem(
        db: &DatabaseConnection,
        id: i64,
    ) -> XRoadResult<Option<subsystem::Model>> {
        subsystem::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get subsystem: {e}")))
    }

    /// Lists all subsystems belonging to a member.
    pub async fn list_subsystems(
        db: &DatabaseConnection,
        member_id: i64,
    ) -> XRoadResult<Vec<subsystem::Model>> {
        subsystem::Entity::find()
            .filter(subsystem::Column::MemberId.eq(member_id))
            .all(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list subsystems: {e}")))
    }

    pub async fn list_all_subsystems(
        db: &DatabaseConnection,
    ) -> XRoadResult<Vec<subsystem::Model>> {
        subsystem::Entity::find()
            .all(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list all subsystems: {e}")))
    }

    pub async fn delete_subsystem(db: &DatabaseConnection, id: i64) -> XRoadResult<bool> {
        let res = subsystem::Entity::delete_by_id(id)
            .exec(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete subsystem: {e}")))?;
        Ok(res.rows_affected > 0)
    }
}
