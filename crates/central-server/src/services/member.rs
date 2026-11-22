use crate::entities::{global_group, global_group_member, member, security_server, subsystem};
use common::{XRoadError, XRoadResult};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

/// Service for managing Members in Central Server.
#[derive(Clone, Copy, Default, Debug)]
pub struct MemberService;

impl MemberService {
    /// Registers a new X-Road member.
    pub async fn add_member(
        db: &DatabaseConnection,
        instance: &str,
        class: &str,
        code: &str,
        name: &str,
        contact: Option<&str>,
    ) -> XRoadResult<member::Model> {
        let existing = Self::get_member(db, instance, class, code).await?;
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
        .insert(db)
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to insert member: {e}")))
    }

    /// Looks up a member by instance, class, and member code.
    pub async fn get_member(
        db: &DatabaseConnection,
        instance: &str,
        class: &str,
        code: &str,
    ) -> XRoadResult<Option<member::Model>> {
        member::Entity::find()
            .filter(member::Column::XroadInstance.eq(instance))
            .filter(member::Column::MemberClass.eq(class))
            .filter(member::Column::MemberCode.eq(code))
            .one(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query member: {e}")))
    }

    pub async fn get_member_by_id(
        db: &DatabaseConnection,
        id: i64,
    ) -> XRoadResult<Option<member::Model>> {
        member::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query member by ID: {e}")))
    }

    pub async fn update_member_name(
        db: &DatabaseConnection,
        id: i64,
        name: &str,
    ) -> XRoadResult<member::Model> {
        let m = Self::get_member_by_id(db, id)
            .await?
            .ok_or_else(|| XRoadError::Configuration(format!("Member not found: {id}")))?;
        let mut active: member::ActiveModel = m.into();
        active.name = Set(name.to_string());
        active.updated_at = Set(chrono::Utc::now());
        active
            .update(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to update member: {e}")))
    }

    pub async fn delete_member(db: &DatabaseConnection, id: i64) -> XRoadResult<bool> {
        subsystem::Entity::delete_many()
            .filter(subsystem::Column::MemberId.eq(id))
            .exec(db)
            .await
            .map_err(|e| {
                XRoadError::Database(format!("Failed to delete member subsystems: {e}"))
            })?;

        let res = member::Entity::delete_by_id(id)
            .exec(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete member: {e}")))?;
        Ok(res.rows_affected > 0)
    }

    /// Lists all members.
    pub async fn list_members(db: &DatabaseConnection) -> XRoadResult<Vec<member::Model>> {
        member::Entity::find()
            .all(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list members: {e}")))
    }

    pub async fn list_member_global_groups(
        db: &DatabaseConnection,
        member_id: i64,
    ) -> XRoadResult<Vec<String>> {
        let member = Self::get_member_by_id(db, member_id)
            .await?
            .ok_or_else(|| XRoadError::Configuration(format!("Member not found: {member_id}")))?;
        let member_prefix = format!(
            "{}/{}/{}",
            member.xroad_instance, member.member_class, member.member_code
        );

        let group_members = global_group_member::Entity::find()
            .all(db)
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
            if let Ok(Some(g)) = global_group::Entity::find_by_id(gid).one(db).await {
                groups.push(g.group_code);
            }
        }
        Ok(groups)
    }

    pub async fn list_member_owned_servers(
        db: &DatabaseConnection,
        member_id: i64,
    ) -> XRoadResult<Vec<security_server::Model>> {
        security_server::Entity::find()
            .filter(security_server::Column::OwnerId.eq(member_id))
            .all(db)
            .await
            .map_err(|e| {
                XRoadError::Database(format!("Failed to query member security servers: {e}"))
            })
    }
}
