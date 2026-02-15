use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum TimelineCommands {
    /// Rebuild all timeline events from current entity data
    Rebuild,
}
