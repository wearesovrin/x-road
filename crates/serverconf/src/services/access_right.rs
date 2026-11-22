use crate::entities::access_right;
use common::{XRoadError, XRoadResult};
use identifiers::{ClientId, ServiceId};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

/// Service for managing service access rights in Security Server configuration.
#[derive(Clone)]
pub struct AccessRightService {
    db: Arc<DatabaseConnection>,
}

impl AccessRightService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn raw_connection(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Grants access for a client to invoke a service.
    pub async fn grant_access(
        &self,
        service_id: &ServiceId,
        client_id: &ClientId,
    ) -> XRoadResult<()> {
        let service_id_str = service_id.to_string();
        let client_id_str = client_id.to_string();

        let existing = access_right::Entity::find()
            .filter(access_right::Column::ServiceIdStr.eq(&service_id_str))
            .filter(access_right::Column::SubjectClientIdStr.eq(&client_id_str))
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Access right query failed: {e}")))?;

        if existing.is_none() {
            access_right::ActiveModel {
                service_id_str: Set(service_id_str),
                subject_client_id_str: Set(client_id_str),
                rights_given: Set(chrono::Utc::now()),
                ..Default::default()
            }
            .insert(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to grant access: {e}")))?;
        }

        Ok(())
    }

    /// Revokes access for a client to invoke a service.
    pub async fn revoke_access(
        &self,
        service_id: &ServiceId,
        client_id: &ClientId,
    ) -> XRoadResult<()> {
        access_right::Entity::delete_many()
            .filter(access_right::Column::ServiceIdStr.eq(service_id.to_string()))
            .filter(access_right::Column::SubjectClientIdStr.eq(client_id.to_string()))
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to revoke access: {e}")))?;

        Ok(())
    }

    /// Lists subjects (clients) with access rights to a service.
    pub async fn list_service_access_rights(
        &self,
        service_id_str: &str,
    ) -> XRoadResult<Vec<String>> {
        let rights = access_right::Entity::find()
            .filter(access_right::Column::ServiceIdStr.eq(service_id_str))
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query access rights: {e}")))?;

        Ok(rights
            .into_iter()
            .map(|r| r.subject_client_id_str)
            .collect())
    }
}
