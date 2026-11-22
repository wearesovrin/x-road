use crate::entities::{global_group, global_group_member};
use common::{XRoadError, XRoadResult};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

/// Service for managing Global Groups in Central Server.
#[derive(Clone, Copy, Default, Debug)]
pub struct GlobalGroupService;

impl GlobalGroupService {
    /// Creates a global group.
    pub async fn create_global_group(
        db: &DatabaseConnection,
        group_code: &str,
        description: &str,
    ) -> XRoadResult<global_group::Model> {
        let now = chrono::Utc::now();
        global_group::ActiveModel {
            group_code: Set(group_code.to_string()),
            description: Set(description.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to create global group: {e}")))
    }

    pub async fn get_global_group(
        db: &DatabaseConnection,
        group_code: &str,
    ) -> XRoadResult<Option<global_group::Model>> {
        global_group::Entity::find()
            .filter(global_group::Column::GroupCode.eq(group_code))
            .one(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get global group: {e}")))
    }

    pub async fn list_global_groups(
        db: &DatabaseConnection,
    ) -> XRoadResult<Vec<global_group::Model>> {
        global_group::Entity::find()
            .all(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list global groups: {e}")))
    }

    pub async fn delete_global_group(
        db: &DatabaseConnection,
        group_code: &str,
    ) -> XRoadResult<bool> {
        if let Some(g) = Self::get_global_group(db, group_code).await? {
            global_group_member::Entity::delete_many()
                .filter(global_group_member::Column::GlobalGroupId.eq(g.id))
                .exec(db)
                .await
                .map_err(|e| {
                    XRoadError::Database(format!("Failed to delete group members: {e}"))
                })?;

            let res = global_group::Entity::delete_by_id(g.id)
                .exec(db)
                .await
                .map_err(|e| XRoadError::Database(format!("Failed to delete global group: {e}")))?;
            Ok(res.rows_affected > 0)
        } else {
            Ok(false)
        }
    }

    /// Adds an identifier to a global group.
    pub async fn add_to_global_group(
        db: &DatabaseConnection,
        group_code: &str,
        identifier: &str,
    ) -> XRoadResult<()> {
        let group = global_group::Entity::find()
            .filter(global_group::Column::GroupCode.eq(group_code))
            .one(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query global group: {e}")))?
            .ok_or_else(|| {
                XRoadError::Configuration(format!("Global group not found: {group_code}"))
            })?;

        global_group_member::ActiveModel {
            global_group_id: Set(group.id),
            identifier: Set(identifier.to_string()),
            created_at: Set(chrono::Utc::now()),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to add member to global group: {e}")))?;

        Ok(())
    }

    pub async fn remove_from_global_group(
        db: &DatabaseConnection,
        group_code: &str,
        identifier: &str,
    ) -> XRoadResult<bool> {
        if let Some(g) = Self::get_global_group(db, group_code).await? {
            let res = global_group_member::Entity::delete_many()
                .filter(global_group_member::Column::GlobalGroupId.eq(g.id))
                .filter(global_group_member::Column::Identifier.eq(identifier))
                .exec(db)
                .await
                .map_err(|e| {
                    XRoadError::Database(format!("Failed to remove member from global group: {e}"))
                })?;
            Ok(res.rows_affected > 0)
        } else {
            Ok(false)
        }
    }

    /// Lists all member identifiers in a global group.
    pub async fn list_global_group_members(
        db: &DatabaseConnection,
        group_code: &str,
    ) -> XRoadResult<Vec<String>> {
        let group = global_group::Entity::find()
            .filter(global_group::Column::GroupCode.eq(group_code))
            .one(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query global group: {e}")))?
            .ok_or_else(|| {
                XRoadError::Configuration(format!("Global group not found: {group_code}"))
            })?;

        let members = global_group_member::Entity::find()
            .filter(global_group_member::Column::GlobalGroupId.eq(group.id))
            .all(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list group members: {e}")))?;

        Ok(members.into_iter().map(|m| m.identifier).collect())
    }
}
