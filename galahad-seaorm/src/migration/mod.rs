use sea_orm_migration::prelude::*;

mod m20260828_000001_create_auth_tables;

/// Runs Galahad's SeaORM migrations.
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20260828_000001_create_auth_tables::Migration)]
    }
}
