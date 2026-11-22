use crate::entities::{member, subsystem};
use common::XRoadResult;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClientItem {
    pub client_id: String,
    pub member_name: String,
    pub member_class: String,
    pub member_code: String,
    pub subsystem_code: Option<String>,
    pub status: String,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct ClientService;

impl ClientService {
    pub async fn list_all_clients(db: &DatabaseConnection) -> XRoadResult<Vec<ClientItem>> {
        let members = member::Entity::find()
            .all(db)
            .await
            .map_err(|e| common::XRoadError::Database(format!("Failed to list members: {e}")))?;

        let mut clients = Vec::new();

        for m in members {
            let member_id_str =
                format!("{}/{}/{}", m.xroad_instance, m.member_class, m.member_code);
            clients.push(ClientItem {
                client_id: member_id_str,
                member_name: m.name.clone(),
                member_class: m.member_class.clone(),
                member_code: m.member_code.clone(),
                subsystem_code: None,
                status: "REGISTERED".to_string(),
            });

            let subs = subsystem::Entity::find()
                .filter(subsystem::Column::MemberId.eq(m.id))
                .all(db)
                .await
                .map_err(|e| {
                    common::XRoadError::Database(format!("Failed to list subsystems: {e}"))
                })?;

            for s in subs {
                let sub_id_str = format!(
                    "{}/{}/{}/{}",
                    m.xroad_instance, m.member_class, m.member_code, s.subsystem_code
                );
                clients.push(ClientItem {
                    client_id: sub_id_str,
                    member_name: m.name.clone(),
                    member_class: m.member_class.clone(),
                    member_code: m.member_code.clone(),
                    subsystem_code: Some(s.subsystem_code),
                    status: "REGISTERED".to_string(),
                });
            }
        }

        Ok(clients)
    }
}
