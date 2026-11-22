use crate::entities::{
    access_right, certificate, client, endpoint, group_member, service, service_description,
};
use common::{XRoadError, XRoadResult};
use identifiers::ClientId;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

/// Service for managing clients in Security Server configuration.
#[derive(Clone)]
pub struct ClientService {
    db: Arc<DatabaseConnection>,
}

impl ClientService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn raw_connection(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Registers a client in the server configuration.
    pub async fn add_client(
        &self,
        client_id: &ClientId,
        is_auth: bool,
    ) -> XRoadResult<client::Model> {
        let existing = self.get_client(client_id).await?;
        if let Some(c) = existing {
            return Ok(c);
        }

        let new_client = client::ActiveModel {
            xroad_instance: Set(client_id.xroad_instance.clone()),
            member_class: Set(client_id.member_class.clone()),
            member_code: Set(client_id.member_code.clone()),
            subsystem_code: Set(client_id.subsystem_code.clone()),
            client_status: Set("REGISTERED".to_string()),
            is_authentication: Set(is_auth),
            ..Default::default()
        };

        new_client
            .insert(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to register client: {e}")))
    }

    /// Look up client record by identifier.
    pub async fn get_client(&self, client_id: &ClientId) -> XRoadResult<Option<client::Model>> {
        let mut query = client::Entity::find()
            .filter(client::Column::XroadInstance.eq(&client_id.xroad_instance))
            .filter(client::Column::MemberClass.eq(&client_id.member_class))
            .filter(client::Column::MemberCode.eq(&client_id.member_code));

        if let Some(ref subsystem) = client_id.subsystem_code {
            query = query.filter(client::Column::SubsystemCode.eq(Some(subsystem.clone())));
        } else {
            query = query.filter(client::Column::SubsystemCode.is_null());
        }

        query
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query client: {e}")))
    }

    /// Lists all registered clients.
    pub async fn list_clients(&self) -> XRoadResult<Vec<ClientId>> {
        let clients = client::Entity::find()
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list clients: {e}")))?;

        let mut result = Vec::new();
        for c in clients {
            let id = if let Some(sub) = c.subsystem_code {
                ClientId::subsystem_unchecked(c.xroad_instance, c.member_class, c.member_code, sub)
            } else {
                ClientId::member_unchecked(c.xroad_instance, c.member_class, c.member_code)
            };
            result.push(id);
        }

        Ok(result)
    }

    /// Deletes a client and all cascading dependencies (services, endpoints, access rights, certificates).
    pub async fn delete_client(&self, client_id: &ClientId) -> XRoadResult<()> {
        let client_model = self.get_client(client_id).await?;
        if let Some(c) = client_model {
            let client_str = client_id.to_string();

            // 1. Delete endpoints and endpoint access rights
            let endpoints = endpoint::Entity::find()
                .filter(endpoint::Column::ClientId.eq(c.id))
                .all(self.raw_connection())
                .await
                .unwrap_or_default();
            for ep in endpoints {
                let ep_ref = format!("EP:{}:{}:{}", ep.service_code, ep.method, ep.path);
                let _ = access_right::Entity::delete_many()
                    .filter(access_right::Column::ServiceIdStr.eq(ep_ref))
                    .exec(self.raw_connection())
                    .await;
                let _ = endpoint::Entity::delete_by_id(ep.id)
                    .exec(self.raw_connection())
                    .await;
            }

            // 2. Delete services and access rights under client's service descriptions
            let descriptions = service_description::Entity::find()
                .filter(service_description::Column::ClientId.eq(c.id))
                .all(self.raw_connection())
                .await
                .unwrap_or_default();

            for desc in &descriptions {
                let services = service::Entity::find()
                    .filter(service::Column::ServiceDescriptionId.eq(desc.id))
                    .all(self.raw_connection())
                    .await
                    .unwrap_or_default();
                for svc in services {
                    let svc_prefix = format!("{}/{}", client_str, svc.service_code);
                    let _ = access_right::Entity::delete_many()
                        .filter(access_right::Column::ServiceIdStr.contains(&svc_prefix))
                        .exec(self.raw_connection())
                        .await;
                    let _ = service::Entity::delete_by_id(svc.id)
                        .exec(self.raw_connection())
                        .await;
                }
            }

            // 3. Delete access rights granted to this client as subject
            let _ = access_right::Entity::delete_many()
                .filter(access_right::Column::SubjectClientIdStr.eq(&client_str))
                .exec(self.raw_connection())
                .await;

            // 4. Delete service descriptions
            let _ = service_description::Entity::delete_many()
                .filter(service_description::Column::ClientId.eq(c.id))
                .exec(self.raw_connection())
                .await;

            // 5. Delete client certificates
            let _ = certificate::Entity::delete_many()
                .filter(certificate::Column::ClientId.eq(c.id))
                .exec(self.raw_connection())
                .await;

            // 6. Delete group memberships
            let _ = group_member::Entity::delete_many()
                .filter(group_member::Column::MemberClientIdStr.eq(&client_str))
                .exec(self.raw_connection())
                .await;

            // 7. Delete client record
            client::Entity::delete_by_id(c.id)
                .exec(self.raw_connection())
                .await
                .map_err(|e| XRoadError::Database(format!("Failed to delete client: {e}")))?;
        }
        Ok(())
    }

    /// Updates status of a client (e.g., "REGISTERED", "DISABLED", "SAVED").
    pub async fn set_client_status(&self, client_id: &ClientId, status: &str) -> XRoadResult<()> {
        let client_model = self
            .get_client(client_id)
            .await?
            .ok_or_else(|| XRoadError::Configuration(format!("Client not found: {client_id}")))?;

        let mut active: client::ActiveModel = client_model.into();
        active.client_status = Set(status.to_string());
        active
            .update(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to update client status: {e}")))?;
        Ok(())
    }

    /// Designates a client as the Security Server owner.
    pub async fn make_owner(&self, client_id: &ClientId) -> XRoadResult<()> {
        let client_model = self
            .get_client(client_id)
            .await?
            .ok_or_else(|| XRoadError::Configuration(format!("Client not found: {client_id}")))?;

        let mut active: client::ActiveModel = client_model.into();
        active.is_authentication = Set(true);
        active
            .update(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to set owner client: {e}")))?;
        Ok(())
    }

    /// Gets the current Security Server owner client.
    pub async fn get_owner(&self) -> XRoadResult<Option<client::Model>> {
        client::Entity::find()
            .filter(client::Column::IsAuthentication.eq(true))
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query owner client: {e}")))
    }
}
