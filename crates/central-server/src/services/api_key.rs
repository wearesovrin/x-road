use crate::entities::{api_key, api_key_role};
use common::{XRoadError, XRoadResult};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// Service for managing Central Server API keys and their assigned roles.
#[derive(Clone)]
pub struct ApiKeyService {
    db: Arc<DatabaseConnection>,
}

impl ApiKeyService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn raw_connection(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Creates an API key with assigned roles.
    pub async fn create_api_key(
        &self,
        encoded_key: &str,
        label: Option<&str>,
        roles: &[&str],
    ) -> XRoadResult<String> {
        let key_id = uuid::Uuid::new_v4().to_string();

        let key_model = api_key::ActiveModel {
            key_id: Set(key_id.clone()),
            encoded_key: Set(encoded_key.to_string()),
            label: Set(label.map(ToString::to_string)),
            ..Default::default()
        }
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to insert API key: {e}")))?;

        for role in roles {
            api_key_role::ActiveModel {
                api_key_id: Set(key_model.id),
                role: Set(role.to_string()),
                ..Default::default()
            }
            .insert(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to insert API key role: {e}")))?;
        }

        Ok(key_id)
    }

    /// Validates an API key and returns its assigned roles if valid.
    /// Checks both plaintext token (hashed with SHA-256) and direct encoded key.
    pub async fn validate_api_key(&self, key_or_token: &str) -> XRoadResult<Option<Vec<String>>> {
        let hashed_token = format!("{:x}", Sha256::digest(key_or_token.as_bytes()));
        let key = api_key::Entity::find()
            .filter(
                sea_orm::Condition::any()
                    .add(api_key::Column::EncodedKey.eq(key_or_token))
                    .add(api_key::Column::EncodedKey.eq(&hashed_token)),
            )
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to validate API key: {e}")))?;

        if let Some(key_model) = key {
            let roles = api_key_role::Entity::find()
                .filter(api_key_role::Column::ApiKeyId.eq(key_model.id))
                .all(self.raw_connection())
                .await
                .map_err(|e| XRoadError::Database(format!("Failed to query API key roles: {e}")))?;

            Ok(Some(roles.into_iter().map(|r| r.role).collect()))
        } else {
            Ok(None)
        }
    }

    /// Lists all registered API keys with their assigned roles.
    pub async fn list_api_keys(&self) -> XRoadResult<Vec<(api_key::Model, Vec<String>)>> {
        let keys = api_key::Entity::find()
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list API keys: {e}")))?;

        let mut result = Vec::with_capacity(keys.len());
        for key in keys {
            let roles = api_key_role::Entity::find()
                .filter(api_key_role::Column::ApiKeyId.eq(key.id))
                .all(self.raw_connection())
                .await
                .map_err(|e| XRoadError::Database(format!("Failed to query API key roles: {e}")))?;
            result.push((key, roles.into_iter().map(|r| r.role).collect()));
        }
        Ok(result)
    }

    /// Gets an API key and its assigned roles by its database ID.
    pub async fn get_api_key(&self, id: i64) -> XRoadResult<Option<(api_key::Model, Vec<String>)>> {
        let key = api_key::Entity::find_by_id(id)
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get API key: {e}")))?;

        if let Some(key_model) = key {
            let roles = api_key_role::Entity::find()
                .filter(api_key_role::Column::ApiKeyId.eq(key_model.id))
                .all(self.raw_connection())
                .await
                .map_err(|e| XRoadError::Database(format!("Failed to query API key roles: {e}")))?;
            Ok(Some((
                key_model,
                roles.into_iter().map(|r| r.role).collect(),
            )))
        } else {
            Ok(None)
        }
    }

    /// Updates the assigned roles of an API key.
    pub async fn update_api_key_roles(&self, id: i64, roles: &[&str]) -> XRoadResult<()> {
        let _key = api_key::Entity::find_by_id(id)
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to find API key: {e}")))?
            .ok_or_else(|| XRoadError::Configuration(format!("API key {id} not found")))?;

        // Remove old roles
        api_key_role::Entity::delete_many()
            .filter(api_key_role::Column::ApiKeyId.eq(id))
            .exec(self.raw_connection())
            .await
            .map_err(|e| {
                XRoadError::Database(format!("Failed to delete old API key roles: {e}"))
            })?;

        // Insert new roles
        for role in roles {
            api_key_role::ActiveModel {
                api_key_id: Set(id),
                role: Set(role.to_string()),
                ..Default::default()
            }
            .insert(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to insert API key role: {e}")))?;
        }

        Ok(())
    }

    /// Revokes (deletes) an API key by its database ID.
    pub async fn revoke_api_key(&self, id: i64) -> XRoadResult<()> {
        api_key_role::Entity::delete_many()
            .filter(api_key_role::Column::ApiKeyId.eq(id))
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete API key roles: {e}")))?;

        api_key::Entity::delete_by_id(id)
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete API key: {e}")))?;

        Ok(())
    }
}
