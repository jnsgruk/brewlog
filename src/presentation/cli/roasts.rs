use anyhow::Result;
use clap::{Args, Subcommand};

use super::macros::{define_delete_command, define_get_command};
use super::parse_created_at;
use super::print_json;
use crate::domain::ids::{RoastId, RoasterId};
use crate::domain::roasts::{NewRoast, UpdateRoast};
use crate::infrastructure::client::BrewlogClient;

#[derive(Debug, Subcommand)]
pub enum RoastCommands {
    /// Add a new roast
    Add(AddRoastCommand),
    /// List all roasts
    List(ListRoastsCommand),
    /// Get a roast by ID
    Get(GetRoastCommand),
    /// Update a roast
    Update(UpdateRoastCommand),
    /// Delete a roast
    Delete(DeleteRoastCommand),
}

pub async fn run(client: &BrewlogClient, cmd: RoastCommands) -> Result<()> {
    match cmd {
        RoastCommands::Add(c) => add_roast(client, c).await,
        RoastCommands::List(c) => list_roasts(client, c).await,
        RoastCommands::Get(c) => get_roast(client, c).await,
        RoastCommands::Update(c) => update_roast(client, c).await,
        RoastCommands::Delete(c) => delete_roast(client, c).await,
    }
}

#[derive(Debug, Args)]
pub struct AddRoastCommand {
    #[arg(long)]
    pub roaster_id: i64,
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub origin: String,
    #[arg(long)]
    pub region: String,
    #[arg(long)]
    pub producer: String,
    #[arg(long)]
    pub process: String,
    #[arg(long = "tasting-notes", required = true)]
    pub tasting_notes: Vec<String>,
    /// Override creation timestamp (e.g. 2025-08-05T10:00:00Z or 2025-08-05)
    #[arg(long)]
    pub created_at: Option<String>,
}

pub async fn add_roast(client: &BrewlogClient, command: AddRoastCommand) -> Result<()> {
    let created_at = command
        .created_at
        .map(|s| parse_created_at(&s))
        .transpose()?;
    let payload = NewRoast {
        roaster_id: RoasterId::new(command.roaster_id),
        name: command.name,
        origin: command.origin,
        region: command.region,
        producer: command.producer,
        tasting_notes: command.tasting_notes,
        process: command.process,
        created_at,
    };

    let roast = client.roasts().create(&payload).await?;
    print_json(&roast)
}

#[derive(Debug, Args)]
pub struct ListRoastsCommand {
    #[arg(long)]
    pub roaster_id: Option<i64>,
}

pub async fn list_roasts(client: &BrewlogClient, command: ListRoastsCommand) -> Result<()> {
    let roasts = client
        .roasts()
        .list(command.roaster_id.map(RoasterId::new))
        .await?;
    print_json(&roasts)
}

define_get_command!(GetRoastCommand, get_roast, RoastId, roasts);

#[derive(Debug, Args)]
pub struct UpdateRoastCommand {
    #[arg(long)]
    pub id: i64,
    #[arg(long)]
    pub roaster_id: Option<i64>,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub origin: Option<String>,
    #[arg(long)]
    pub region: Option<String>,
    #[arg(long)]
    pub producer: Option<String>,
    #[arg(long)]
    pub process: Option<String>,
    #[arg(long = "tasting-notes")]
    pub tasting_notes: Option<Vec<String>>,
    /// Override creation timestamp (e.g. 2025-08-05T10:00:00Z or 2025-08-05)
    #[arg(long)]
    pub created_at: Option<String>,
}

pub async fn update_roast(client: &BrewlogClient, command: UpdateRoastCommand) -> Result<()> {
    let created_at = command
        .created_at
        .map(|s| parse_created_at(&s))
        .transpose()?;
    let payload = UpdateRoast {
        roaster_id: command.roaster_id.map(RoasterId::new),
        name: command.name,
        origin: command.origin,
        region: command.region,
        producer: command.producer,
        tasting_notes: command.tasting_notes,
        process: command.process,
        created_at,
    };

    let roast = client
        .roasts()
        .update(RoastId::new(command.id), &payload)
        .await?;
    print_json(&roast)
}

define_delete_command!(DeleteRoastCommand, delete_roast, RoastId, roasts, "roast");
