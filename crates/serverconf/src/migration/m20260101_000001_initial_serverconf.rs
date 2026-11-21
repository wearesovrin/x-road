use sea_orm::Schema;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let builder = manager.get_database_backend();
        let schema = Schema::new(builder);

        manager
            .create_table(schema.create_table_from_entity(crate::entities::client::Entity))
            .await?;
        manager
            .create_table(schema.create_table_from_entity(crate::entities::certificate::Entity))
            .await?;
        manager
            .create_table(schema.create_table_from_entity(crate::entities::service_description::Entity))
            .await?;
        manager
            .create_table(schema.create_table_from_entity(crate::entities::service::Entity))
            .await?;
        manager
            .create_table(schema.create_table_from_entity(crate::entities::endpoint::Entity))
            .await?;
        manager
            .create_table(schema.create_table_from_entity(crate::entities::access_right::Entity))
            .await?;
        manager
            .create_table(schema.create_table_from_entity(crate::entities::api_key::Entity))
            .await?;
        manager
            .create_table(schema.create_table_from_entity(crate::entities::api_key_role::Entity))
            .await?;
        manager
            .create_table(schema.create_table_from_entity(crate::entities::local_group::Entity))
            .await?;
        manager
            .create_table(schema.create_table_from_entity(crate::entities::group_member::Entity))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(crate::entities::group_member::Entity).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(crate::entities::local_group::Entity).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(crate::entities::api_key_role::Entity).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(crate::entities::api_key::Entity).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(crate::entities::access_right::Entity).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(crate::entities::endpoint::Entity).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(crate::entities::service::Entity).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(crate::entities::service_description::Entity).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(crate::entities::certificate::Entity).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(crate::entities::client::Entity).to_owned())
            .await?;

        Ok(())
    }
}
