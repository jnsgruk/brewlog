use anyhow::Result;
use clap::{Args, Subcommand};

use super::macros::{define_delete_command, define_get_command};
use super::print_json;
use crate::domain::cups::NewCup;
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
    /// Delete a cup
    Delete(DeleteCupCommand),
}

pub async fn run(client: &BrewlogClient, cmd: CupCommands) -> Result<()> {
    match cmd {
        CupCommands::Add(c) => add_cup(client, c).await,
        CupCommands::List => list_cups(client).await,
        CupCommands::Get(c) => get_cup(client, c).await,
        CupCommands::Delete(c) => delete_cup(client, c).await,
    }
}

#[derive(Debug, Args)]
pub struct AddCupCommand {
    #[arg(long)]
    pub roast_id: i64,
    #[arg(long)]
    pub cafe_id: i64,
}

pub async fn add_cup(client: &BrewlogClient, command: AddCupCommand) -> Result<()> {
    let payload = NewCup {
        roast_id: RoastId::new(command.roast_id),
        cafe_id: CafeId::new(command.cafe_id),
    };

    let cup = client.cups().create(&payload).await?;
    print_json(&cup)
}

pub async fn list_cups(client: &BrewlogClient) -> Result<()> {
    let cups = client.cups().list().await?;
    print_json(&cups)
}

define_get_command!(GetCupCommand, get_cup, CupId, cups);
define_delete_command!(DeleteCupCommand, delete_cup, CupId, cups, "cup");
