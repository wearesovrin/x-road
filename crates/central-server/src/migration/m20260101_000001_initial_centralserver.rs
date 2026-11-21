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
            .create_table(schema.create_table_from_entity(crate::entities::member::Entity))
            .await?;
        manager
            .create_table(schema.create_table_from_entity(crate::entities::subsystem::Entity))
            .await?;
        manager
            .create_table(schema.create_table_from_entity(crate::entities::security_server::Entity))
            .await?;
        manager
            .create_table(schema.create_table_from_entity(crate::entities::security_server_client::Entity))
            .await?;
        manager
            .create_table(schema.create_table_from_entity(crate::entities::global_group::Entity))
            .await?;
        manager
            .create_table(schema.create_table_from_entity(crate::entities::global_group_member::Entity))
            .await?;
        manager
            .create_table(schema.create_table_from_entity(crate::entities::approved_ca::Entity))
            .await?;
        manager
            .create_table(schema.create_table_from_entity(crate::entities::approved_tsa::Entity))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(crate::entities::approved_tsa::Entity).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(crate::entities::approved_ca::Entity).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(crate::entities::global_group_member::Entity).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(crate::entities::global_group::Entity).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(crate::entities::security_server_client::Entity).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(crate::entities::security_server::Entity).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(crate::entities::subsystem::Entity).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(crate::entities::member::Entity).to_owned())
            .await?;

        Ok(())
    }
}
