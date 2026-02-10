use anyhow::Result;
use clap::{Args, Subcommand};

use super::macros::{define_delete_command, define_get_command};
use super::parse_created_at;
use super::print_json;
use crate::domain::cups::{NewCup, UpdateCup};
use crate::domain::ids::{CafeId, CupId, RoastId};
use crate::infrastructure::client::BrewlogClient;

#[derive(Debug, Subcommand)]
pub enum CupCommands {
    /// Add a new cup
    Add(AddCupCommand),
    /// List all cups
    List,
    /// Get a cup by ID
    Get(GetCupCommand),
    /// Update a cup
    Update(UpdateCupCommand),
    /// Delete a cup
    Delete(DeleteCupCommand),
}

pub async fn run(client: &BrewlogClient, cmd: CupCommands) -> Result<()> {
    match cmd {
        CupCommands::Add(c) => add_cup(client, c).await,
        CupCommands::List => list_cups(client).await,
        CupCommands::Get(c) => get_cup(client, c).await,
        CupCommands::Update(c) => update_cup(client, c).await,
        CupCommands::Delete(c) => delete_cup(client, c).await,
    }
}

#[derive(Debug, Args)]
pub struct AddCupCommand {
    #[arg(long)]
    pub roast_id: i64,
    #[arg(long)]
    pub cafe_id: i64,
    /// Override creation timestamp (e.g. 2025-08-05T10:00:00Z or 2025-08-05)
    #[arg(long)]
    pub created_at: Option<String>,
}

pub async fn add_cup(client: &BrewlogClient, command: AddCupCommand) -> Result<()> {
    let created_at = command
        .created_at
        .map(|s| parse_created_at(&s))
        .transpose()?;
    let payload = NewCup {
        roast_id: RoastId::new(command.roast_id),
        cafe_id: CafeId::new(command.cafe_id),
        created_at,
    };

    let cup = client.cups().create(&payload).await?;
    print_json(&cup)
}

pub async fn list_cups(client: &BrewlogClient) -> Result<()> {
    let cups = client.cups().list().await?;
    print_json(&cups)
}

#[derive(Debug, Args)]
pub struct UpdateCupCommand {
    #[arg(long)]
    pub id: i64,

    /// ID of the roast
    #[arg(long)]
    pub roast_id: Option<i64>,

    /// ID of the cafe
    #[arg(long)]
    pub cafe_id: Option<i64>,

    /// Override creation timestamp (e.g. 2025-08-05T10:00:00Z or 2025-08-05)
    #[arg(long)]
    pub created_at: Option<String>,
}

pub async fn update_cup(client: &BrewlogClient, command: UpdateCupCommand) -> Result<()> {
    let created_at = command
        .created_at
        .map(|s| parse_created_at(&s))
        .transpose()?;
    let payload = UpdateCup {
        roast_id: command.roast_id.map(RoastId::new),
        cafe_id: command.cafe_id.map(CafeId::new),
        created_at,
    };

    let cup = client
        .cups()
        .update(CupId::new(command.id), &payload)
        .await?;
    print_json(&cup)
}

define_get_command!(GetCupCommand, get_cup, CupId, cups);
define_delete_command!(DeleteCupCommand, delete_cup, CupId, cups, "cup");
