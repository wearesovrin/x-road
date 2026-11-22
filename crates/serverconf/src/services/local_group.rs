use crate::entities::{group_member, local_group};
use crate::services::client::ClientService;
use common::{XRoadError, XRoadResult};
use identifiers::ClientId;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

/// Service for managing local groups and memberships in Security Server configuration.
#[derive(Clone)]
pub struct LocalGroupService {
    db: Arc<DatabaseConnection>,
    clients: ClientService,
}

impl LocalGroupService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            db: db.clone(),
            clients: ClientService::new(db),
        }
    }

    fn raw_connection(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Creates a local group for a client.
    pub async fn create_local_group(
        &self,
        client_id: &ClientId,
        group_code: &str,
        description: &str,
    ) -> XRoadResult<local_group::Model> {
        let client_model =
            self.clients.get_client(client_id).await?.ok_or_else(|| {
                XRoadError::Configuration(format!("Client not found: {client_id}"))
            })?;

        local_group::ActiveModel {
            client_id: Set(client_model.id),
            group_code: Set(group_code.to_string()),
            description: Set(description.to_string()),
            updated: Set(chrono::Utc::now()),
            ..Default::default()
        }
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to create local group: {e}")))
    }

    /// Lists local groups for a client.
    pub async fn list_local_groups(
        &self,
        client_id: &ClientId,
    ) -> XRoadResult<Vec<local_group::Model>> {
        let client_model =
            self.clients.get_client(client_id).await?.ok_or_else(|| {
                XRoadError::Configuration(format!("Client not found: {client_id}"))
            })?;

        local_group::Entity::find()
            .filter(local_group::Column::ClientId.eq(client_model.id))
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query local groups: {e}")))
    }

    /// Gets a local group by ID.
    pub async fn get_local_group(&self, id: i64) -> XRoadResult<Option<local_group::Model>> {
        local_group::Entity::find_by_id(id)
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query local group: {e}")))
    }

    /// Deletes a local group by ID.
    pub async fn delete_local_group(&self, id: i64) -> XRoadResult<()> {
        group_member::Entity::delete_many()
            .filter(group_member::Column::LocalGroupId.eq(id))
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete group members: {e}")))?;

        local_group::Entity::delete_by_id(id)
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete local group: {e}")))?;
        Ok(())
    }

    /// Adds a member client to a local group.
    pub async fn add_group_member(&self, group_id: i64, member_id: &ClientId) -> XRoadResult<()> {
        let member_str = member_id.to_string();
        let existing = group_member::Entity::find()
            .filter(group_member::Column::LocalGroupId.eq(group_id))
            .filter(group_member::Column::MemberClientIdStr.eq(&member_str))
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query group member: {e}")))?;

        if existing.is_none() {
            group_member::ActiveModel {
                local_group_id: Set(group_id),
                member_client_id_str: Set(member_str),
                added: Set(chrono::Utc::now()),
                ..Default::default()
            }
            .insert(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to add group member: {e}")))?;
        }
        Ok(())
    }

    /// Removes a member client from a local group.
    pub async fn remove_group_member(
        &self,
        group_id: i64,
        member_id: &ClientId,
    ) -> XRoadResult<()> {
        let member_str = member_id.to_string();
        group_member::Entity::delete_many()
            .filter(group_member::Column::LocalGroupId.eq(group_id))
            .filter(group_member::Column::MemberClientIdStr.eq(member_str))
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to remove group member: {e}")))?;
        Ok(())
    }

    /// Lists member clients of a local group.
    pub async fn list_group_members(&self, group_id: i64) -> XRoadResult<Vec<String>> {
        let members = group_member::Entity::find()
            .filter(group_member::Column::LocalGroupId.eq(group_id))
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query group members: {e}")))?;

        Ok(members
            .into_iter()
            .map(|m| m.member_client_id_str)
            .collect())
    }
}
