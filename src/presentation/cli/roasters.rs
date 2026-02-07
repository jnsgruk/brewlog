use anyhow::Result;
use clap::{Args, Subcommand};

use super::macros::{define_delete_command, define_get_command};
use super::parse_created_at;
use super::print_json;
use crate::domain::ids::RoasterId;
use crate::domain::roasters::{NewRoaster, UpdateRoaster};
use crate::infrastructure::client::BrewlogClient;

#[derive(Debug, Subcommand)]
pub enum RoasterCommands {
    /// Add a new roaster
    Add(AddRoasterCommand),
    /// List all roasters
    List,
    /// Get a roaster by ID
    Get(GetRoasterCommand),
    /// Update a roaster
    Update(UpdateRoasterCommand),
    /// Delete a roaster
    Delete(DeleteRoasterCommand),
}

pub async fn run(client: &BrewlogClient, cmd: RoasterCommands) -> Result<()> {
    match cmd {
        RoasterCommands::Add(c) => add_roaster(client, c).await,
        RoasterCommands::List => list_roasters(client).await,
        RoasterCommands::Get(c) => get_roaster(client, c).await,
        RoasterCommands::Update(c) => update_roaster(client, c).await,
        RoasterCommands::Delete(c) => delete_roaster(client, c).await,
    }
}

#[derive(Debug, Args)]
pub struct AddRoasterCommand {
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub country: String,
    #[arg(long)]
    pub city: Option<String>,
    #[arg(long)]
    pub homepage: Option<String>,
    /// Override creation timestamp (e.g. 2025-08-05T10:00:00Z or 2025-08-05)
    #[arg(long)]
    pub created_at: Option<String>,
}

pub async fn add_roaster(client: &BrewlogClient, command: AddRoasterCommand) -> Result<()> {
    let created_at = command
        .created_at
        .map(|s| parse_created_at(&s))
        .transpose()?;
    let payload = NewRoaster {
        name: command.name,
        country: command.country,
        city: command.city,
        homepage: command.homepage,
        created_at,
    };

    let roaster = client.roasters().create(&payload).await?;
    print_json(&roaster)
}

pub async fn list_roasters(client: &BrewlogClient) -> Result<()> {
    let roasters = client.roasters().list().await?;
    print_json(&roasters)
}

define_get_command!(GetRoasterCommand, get_roaster, RoasterId, roasters);

#[derive(Debug, Args)]
pub struct UpdateRoasterCommand {
    #[arg(long)]
    pub id: i64,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub country: Option<String>,
    #[arg(long)]
    pub city: Option<String>,
    #[arg(long)]
    pub homepage: Option<String>,
    /// Override creation timestamp (e.g. 2025-08-05T10:00:00Z or 2025-08-05)
    #[arg(long)]
    pub created_at: Option<String>,
}

pub async fn update_roaster(client: &BrewlogClient, command: UpdateRoasterCommand) -> Result<()> {
    let created_at = command
        .created_at
        .map(|s| parse_created_at(&s))
        .transpose()?;
    let payload = UpdateRoaster {
        name: command.name,
        country: command.country,
        city: command.city,
        homepage: command.homepage,
        created_at,
    };

    let roaster = client
        .roasters()
        .update(RoasterId::new(command.id), &payload)
        .await?;
    print_json(&roaster)
}

define_delete_command!(
    DeleteRoasterCommand,
    delete_roaster,
    RoasterId,
    roasters,
    "roaster"
);
