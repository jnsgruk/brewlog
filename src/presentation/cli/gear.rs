use anyhow::Result;
use clap::Args;

use super::macros::{define_delete_command, define_get_command};
use super::print_json;
use crate::domain::ids::GearId;
use crate::infrastructure::client::BrewlogClient;

#[derive(Debug, Args)]
pub struct AddGearCommand {
    #[arg(long)]
    pub category: String,
    #[arg(long)]
    pub make: String,
    #[arg(long)]
    pub model: String,
    #[arg(long)]
    pub notes: Option<String>,
}

pub async fn add_gear(client: &BrewlogClient, command: AddGearCommand) -> Result<()> {
    let gear = client
        .gear()
        .create(&command.category, command.make, command.model, command.notes)
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
    #[arg(long)]
    pub notes: Option<String>,
}

pub async fn update_gear(client: &BrewlogClient, command: UpdateGearCommand) -> Result<()> {
    let gear = client
        .gear()
        .update(
            GearId::new(command.id),
            command.make,
            command.model,
            command.notes,
        )
        .await?;
    print_json(&gear)
}

define_delete_command!(DeleteGearCommand, delete_gear, GearId, gear, "gear");
