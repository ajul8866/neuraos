//! # neuraos-migrate
//! Database migration management for NeuraOS.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod migrator;
pub mod runner;
pub mod version;

pub mod version {
    //! Schema version constants.

    /// Current schema version.
    pub const CURRENT_VERSION: u32 = 1;
}

pub mod migrator {
    //! High-level migration runner.
    use anyhow::Result;

    /// Run all pending migrations against the given database URL.
    pub async fn run_migrations(database_url: &str) -> Result<()> {
        tracing::info!("Running migrations against {database_url}");
        // sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(())
    }
}

pub mod runner {
    //! Migration runner utilities.
}
