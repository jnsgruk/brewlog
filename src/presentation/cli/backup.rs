use clap::Args;

#[derive(Debug, Args)]
pub struct BackupCommand;

#[derive(Debug, Args)]
pub struct RestoreCommand {
    /// Path to the backup JSON file
    #[arg(long)]
    pub file: String,
}
