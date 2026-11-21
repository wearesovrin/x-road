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
            .create_table(schema.create_table_from_entity(crate::entities::log_record::Entity))
            .await?;
        manager
            .create_table(schema.create_table_from_entity(crate::entities::attachment::Entity))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(crate::entities::attachment::Entity).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(crate::entities::log_record::Entity).to_owned())
            .await?;

        Ok(())
    }
}
