use clap::Args;

#[derive(Debug, Args)]
pub struct BackupCommand {
    /// Database URL to back up from
    #[arg(
        long,
        env = "BREWLOG_DATABASE_URL",
        default_value = "sqlite://brewlog.db"
    )]
    pub database_url: String,
}

#[derive(Debug, Args)]
pub struct RestoreCommand {
    /// Database URL to restore into (must be an empty database)
    #[arg(
        long,
        env = "BREWLOG_DATABASE_URL",
        default_value = "sqlite://brewlog.db"
    )]
    pub database_url: String,

    /// Path to the backup JSON file
    #[arg(long)]
    pub file: String,
}
