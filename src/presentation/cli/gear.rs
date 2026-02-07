use anyhow::Result;
use clap::{Args, Subcommand};

use super::macros::{define_delete_command, define_get_command};
use super::parse_created_at;
use super::print_json;
use crate::domain::ids::GearId;
use crate::infrastructure::client::BrewlogClient;

#[derive(Debug, Subcommand)]
pub enum GearCommands {
    /// Add new gear
    Add(AddGearCommand),
    /// List all gear
    List(ListGearCommand),
    /// Get gear by ID
    Get(GetGearCommand),
    /// Update gear
    Update(UpdateGearCommand),
    /// Delete gear
    Delete(DeleteGearCommand),
}

pub async fn run(client: &BrewlogClient, cmd: GearCommands) -> Result<()> {
    match cmd {
        GearCommands::Add(c) => add_gear(client, c).await,
        GearCommands::List(c) => list_gear(client, c).await,
        GearCommands::Get(c) => get_gear(client, c).await,
        GearCommands::Update(c) => update_gear(client, c).await,
        GearCommands::Delete(c) => delete_gear(client, c).await,
    }
}

#[derive(Debug, Args)]
pub struct AddGearCommand {
    #[arg(long)]
    pub category: String,
    #[arg(long)]
    pub make: String,
    #[arg(long)]
    pub model: String,
    /// Override creation timestamp (e.g. 2025-08-05T10:00:00Z or 2025-08-05)
    #[arg(long)]
    pub created_at: Option<String>,
}

pub async fn add_gear(client: &BrewlogClient, command: AddGearCommand) -> Result<()> {
    let created_at = command
        .created_at
        .map(|s| parse_created_at(&s))
        .transpose()?;
    let gear = client
        .gear()
        .create(&command.category, command.make, command.model, created_at)
        .await?;
    print_json(&gear)
}

#[derive(Debug, Args)]
pub struct ListGearCommand {
    #[arg(long)]
    pub category: Option<String>,
}

pub async fn list_gear(client: &BrewlogClient, command: ListGearCommand) -> Result<()> {
    let gear = client.gear().list(command.category).await?;
    print_json(&gear)
}

define_get_command!(GetGearCommand, get_gear, GearId, gear);

#[derive(Debug, Args)]
pub struct UpdateGearCommand {
    #[arg(long)]
    pub id: i64,
    #[arg(long)]
    pub make: Option<String>,
    #[arg(long)]
    pub model: Option<String>,
    /// Override creation timestamp (e.g. 2025-08-05T10:00:00Z or 2025-08-05)
    #[arg(long)]
    pub created_at: Option<String>,
}

pub async fn update_gear(client: &BrewlogClient, command: UpdateGearCommand) -> Result<()> {
    let created_at = command
        .created_at
        .map(|s| parse_created_at(&s))
        .transpose()?;
    let gear = client
        .gear()
        .update(
            GearId::new(command.id),
            command.make,
            command.model,
            created_at,
        )
        .await?;
    print_json(&gear)
}

define_delete_command!(DeleteGearCommand, delete_gear, GearId, gear, "gear");
