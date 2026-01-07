use crate::error::{DatabaseError, DatabaseErrorExt};
use crate::generated::migrations_manifest::{builtin_migrations, builtin_registry};
use fxhash::FxHashMap;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::SurrealValue;

#[derive(Debug, SurrealValue)]
pub(crate) struct Permissions {
    pub slice: &'static str,
    pub permissions: Vec<&'static str>,
}

impl Permissions {
    #[must_use]
    pub(crate) const fn new(slice: &'static str, permissions: Vec<&'static str>) -> Self {
        Self { slice, permissions }
    }
}

#[derive(Debug)]
pub(crate) struct Migration {
    pub slice_key: &'static str,
    pub slice_name: &'static str,
    pub slice_description: Option<&'static str>,
    pub version: &'static str,
    pub script: &'static str,
    pub checksum: &'static str,
    pub is_bootstrap: bool,
}

impl Migration {
    #[must_use]
    pub(crate) const fn new(
        slice_key: &'static str,
        slice_name: &'static str,
        slice_description: Option<&'static str>,
        version: &'static str,
        script: &'static str,
        checksum: &'static str,
        is_bootstrap: bool,
    ) -> Self {
        Self { slice_key, slice_name, slice_description, version, script, checksum, is_bootstrap }
    }

    fn to_applied(&self) -> AppliedMigration {
        AppliedMigration {
            slice_key: self.slice_key.to_owned(),
            version: self.version.to_owned(),
            checksum: self.checksum.to_owned(),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct MigrationReport {
    pub applied: Vec<AppliedMigration>,
    pub skipped: Vec<AppliedMigration>,
}

#[derive(Debug, SurrealValue)]
pub(crate) struct AppliedMigration {
    pub slice_key: String,
    pub version: String,
    pub checksum: String,
}

#[derive(Debug)]
pub(crate) struct MigrationRunner {
    db: Surreal<Any>,
}

impl MigrationRunner {
    #[must_use]
    pub(crate) const fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }
    pub(crate) async fn run(&self) -> Result<MigrationReport, DatabaseError> {
        let mut report = MigrationReport::default();
        let migrations = builtin_migrations();
        let applied_migrations = self.get_migrations_map().await?;

        for migration in migrations {
            if let Some(applied) =
                applied_migrations.get(&format!("{}:{}", migration.slice_key, migration.version))
            {
                ensure_checksum_match(&migration, &applied.checksum)?;
                report.skipped.push(migration.to_applied());
                continue;
            }

            self.apply_migration(&migration).await?;
            report.applied.push(migration.to_applied());
        }

        self.sync_permissions().await?;

        Ok(report)
    }

    async fn apply_migration(&self, migration: &Migration) -> Result<(), DatabaseError> {
        let query = if migration.is_bootstrap {
            format!(
                "BEGIN TRANSACTION;
                {}
                fn::ensure_slice($slice, $name, $desc);
                RETURN fn::confirm_migration($slice, $version, $checksum);
                COMMIT TRANSACTION;",
                migration.script,
            )
        } else {
            format!(
                "BEGIN TRANSACTION;
                fn::ensure_slice($slice, $name, $description);
                {}
                RETURN fn::confirm_migration($slice, $version, $checksum);
                COMMIT TRANSACTION;",
                migration.script,
            )
        };

        let _ = self
            .db
            .query(&query)
            .bind(("slice", migration.slice_key))
            .bind(("name", migration.slice_name))
            .bind(("description", migration.slice_description))
            .bind(("version", migration.version))
            .bind(("checksum", migration.checksum))
            .await
            .context(format!(
                "SQL execution failed at {}:{}",
                migration.slice_key, migration.version
            ))?;

        Ok(())
    }

    async fn is_system_ready(&self) -> Result<bool, DatabaseError> {
        let mut response = self
            .db
            .query("!(SELECT VALUE fields FROM ONLY INFO FOR TABLE slice).is_empty()")
            .await
            .context("Checking if system is ready")?;

        let is_ready = response.take::<Option<bool>>(0)?.unwrap_or_default();
        Ok(is_ready)
    }

    async fn get_migrations_map(
        &self,
    ) -> Result<FxHashMap<String, AppliedMigration>, DatabaseError> {
        let is_ready = self.is_system_ready().await?;

        if !is_ready {
            return Ok(FxHashMap::default());
        }

        let entries = self
            .db
            .query("SELECT id[0].id() as slice_key, version, checksum FROM migration")
            .await
            .context("Loading applied migrations")?
            .take::<Vec<AppliedMigration>>(0)
            .context("Parsing migrations map")?;

        Ok(entries
            .into_iter()
            .map(|entry| (format!("{}:{}", entry.slice_key, entry.version), entry))
            .collect())
    }

    pub(crate) async fn sync_permissions(&self) -> Result<(), DatabaseError> {
        let registry = builtin_registry();

        self.db
            .query("fn::sync_permissions($registry)")
            .bind(("registry", registry))
            .await?
            .check()
            .map_err(surrealdb::Error::from)?;

        Ok(())
    }
}

fn ensure_checksum_match(migration: &Migration, existing: &str) -> Result<(), DatabaseError> {
    if existing != migration.checksum {
        return Err(DatabaseError::Migration {
            message: format!(
                "Checksum mismatch for {}:{} (expected {}, got {})",
                migration.slice_key, migration.version, existing, migration.checksum
            )
            .into(),
            context: Some("Migration already applied with different checksum".into()),
        });
    }
    Ok(())
}
