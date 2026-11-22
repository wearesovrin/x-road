use crate::entities::{global_group, global_group_member, member, security_server, subsystem};
use common::{XRoadError, XRoadResult};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

#[derive(Clone)]
pub struct MemberService {
    db: Arc<DatabaseConnection>,
}

impl MemberService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub fn conn(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Registers a new X-Road member.
    pub async fn add_member(
        &self,
        instance: &str,
        class: &str,
        code: &str,
        name: &str,
        contact: Option<&str>,
    ) -> XRoadResult<member::Model> {
        let existing = self.get_member(instance, class, code).await?;
        if let Some(m) = existing {
            return Ok(m);
        }

        let now = chrono::Utc::now();
        member::ActiveModel {
            xroad_instance: Set(instance.to_string()),
            member_class: Set(class.to_string()),
            member_code: Set(code.to_string()),
            name: Set(name.to_string()),
            administrative_contact: Set(contact.map(ToString::to_string)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(self.conn())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to insert member: {e}")))
    }

    /// Looks up a member by instance, class, and member code.
    pub async fn get_member(
        &self,
        instance: &str,
        class: &str,
        code: &str,
    ) -> XRoadResult<Option<member::Model>> {
        member::Entity::find()
            .filter(member::Column::XroadInstance.eq(instance))
            .filter(member::Column::MemberClass.eq(class))
            .filter(member::Column::MemberCode.eq(code))
            .one(self.conn())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query member: {e}")))
    }

    pub async fn get_member_by_id(&self, id: i64) -> XRoadResult<Option<member::Model>> {
        member::Entity::find_by_id(id)
            .one(self.conn())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query member by ID: {e}")))
    }

    pub async fn update_member_name(&self, id: i64, name: &str) -> XRoadResult<member::Model> {
        let m = self
            .get_member_by_id(id)
            .await?
            .ok_or_else(|| XRoadError::Configuration(format!("Member not found: {id}")))?;
        let mut active: member::ActiveModel = m.into();
        active.name = Set(name.to_string());
        active.updated_at = Set(chrono::Utc::now());
        active
            .update(self.conn())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to update member: {e}")))
    }

    pub async fn delete_member(&self, id: i64) -> XRoadResult<bool> {
        subsystem::Entity::delete_many()
            .filter(subsystem::Column::MemberId.eq(id))
            .exec(self.conn())
            .await
            .map_err(|e| {
                XRoadError::Database(format!("Failed to delete member subsystems: {e}"))
            })?;

        let res = member::Entity::delete_by_id(id)
            .exec(self.conn())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete member: {e}")))?;
        Ok(res.rows_affected > 0)
    }

    /// Lists all members.
    pub async fn list_members(&self) -> XRoadResult<Vec<member::Model>> {
        member::Entity::find()
            .all(self.conn())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list members: {e}")))
    }

    pub async fn list_member_global_groups(&self, member_id: i64) -> XRoadResult<Vec<String>> {
        let member = self
            .get_member_by_id(member_id)
            .await?
            .ok_or_else(|| XRoadError::Configuration(format!("Member not found: {member_id}")))?;
        let member_prefix = format!(
            "{}/{}/{}",
            member.xroad_instance, member.member_class, member.member_code
        );

        let group_members = global_group_member::Entity::find()
            .all(self.conn())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query group members: {e}")))?;

        let mut matched_group_ids = std::collections::HashSet::new();
        for gm in group_members {
            if gm.identifier.starts_with(&member_prefix) {
                matched_group_ids.insert(gm.global_group_id);
            }
        }

        let mut groups = Vec::new();
        for gid in matched_group_ids {
            if let Ok(Some(g)) = global_group::Entity::find_by_id(gid).one(self.conn()).await {
                groups.push(g.group_code);
            }
        }
        Ok(groups)
    }

    pub async fn list_member_owned_servers(
        &self,
        member_id: i64,
    ) -> XRoadResult<Vec<security_server::Model>> {
        security_server::Entity::find()
            .filter(security_server::Column::OwnerId.eq(member_id))
            .all(self.conn())
            .await
            .map_err(|e| {
                XRoadError::Database(format!("Failed to query member security servers: {e}"))
            })
    }
}
