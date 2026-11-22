use crate::entities::{global_group, global_group_member};
use common::{XRoadError, XRoadResult};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

/// Service for managing Global Groups in Central Server.
#[derive(Clone)]
pub struct GlobalGroupService {
    db: Arc<DatabaseConnection>,
}

impl GlobalGroupService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn raw_connection(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Creates a global group.
    pub async fn create_global_group(
        &self,
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
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to create global group: {e}")))
    }

    pub async fn get_global_group(
        &self,
        group_code: &str,
    ) -> XRoadResult<Option<global_group::Model>> {
        global_group::Entity::find()
            .filter(global_group::Column::GroupCode.eq(group_code))
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get global group: {e}")))
    }

    pub async fn list_global_groups(&self) -> XRoadResult<Vec<global_group::Model>> {
        global_group::Entity::find()
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list global groups: {e}")))
    }

    pub async fn delete_global_group(&self, group_code: &str) -> XRoadResult<bool> {
        if let Some(g) = self.get_global_group(group_code).await? {
            global_group_member::Entity::delete_many()
                .filter(global_group_member::Column::GlobalGroupId.eq(g.id))
                .exec(self.raw_connection())
                .await
                .map_err(|e| {
                    XRoadError::Database(format!("Failed to delete group members: {e}"))
                })?;

            let res = global_group::Entity::delete_by_id(g.id)
                .exec(self.raw_connection())
                .await
                .map_err(|e| XRoadError::Database(format!("Failed to delete global group: {e}")))?;
            Ok(res.rows_affected > 0)
        } else {
            Ok(false)
        }
    }

    /// Adds an identifier to a global group.
    pub async fn add_to_global_group(&self, group_code: &str, identifier: &str) -> XRoadResult<()> {
        let group = global_group::Entity::find()
            .filter(global_group::Column::GroupCode.eq(group_code))
            .one(self.raw_connection())
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
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to add member to global group: {e}")))?;

        Ok(())
    }

    pub async fn remove_from_global_group(
        &self,
        group_code: &str,
        identifier: &str,
    ) -> XRoadResult<bool> {
        if let Some(g) = self.get_global_group(group_code).await? {
            let res = global_group_member::Entity::delete_many()
                .filter(global_group_member::Column::GlobalGroupId.eq(g.id))
                .filter(global_group_member::Column::Identifier.eq(identifier))
                .exec(self.raw_connection())
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
    pub async fn list_global_group_members(&self, group_code: &str) -> XRoadResult<Vec<String>> {
        let group = global_group::Entity::find()
            .filter(global_group::Column::GroupCode.eq(group_code))
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query global group: {e}")))?
            .ok_or_else(|| {
                XRoadError::Configuration(format!("Global group not found: {group_code}"))
            })?;

        let members = global_group_member::Entity::find()
            .filter(global_group_member::Column::GlobalGroupId.eq(group.id))
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list group members: {e}")))?;

        Ok(members.into_iter().map(|m| m.identifier).collect())
    }
}
