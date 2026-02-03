use anyhow::Result;
use clap::{Args, Subcommand};

use super::macros::{define_delete_command, define_get_command};
use super::print_json;
use crate::domain::ids::{BagId, BrewId, GearId};
use crate::infrastructure::client::BrewlogClient;

#[derive(Debug, Subcommand)]
pub enum BrewCommands {
    /// Add a new brew
    Add(AddBrewCommand),
    /// List all brews
    List(ListBrewsCommand),
    /// Get a brew by ID
    Get(GetBrewCommand),
    /// Delete a brew
    Delete(DeleteBrewCommand),
}

pub async fn run(client: &BrewlogClient, cmd: BrewCommands) -> Result<()> {
    match cmd {
        BrewCommands::Add(c) => add_brew(client, c).await,
        BrewCommands::List(c) => list_brews(client, c).await,
        BrewCommands::Get(c) => get_brew(client, c).await,
        BrewCommands::Delete(c) => delete_brew(client, c).await,
    }
}

#[derive(Debug, Args)]
pub struct AddBrewCommand {
    /// ID of the bag to brew from
    #[arg(long)]
    pub bag_id: i64,

    /// Amount of coffee in grams
    #[arg(long, default_value = "15.0")]
    pub coffee_weight: f64,

    /// ID of the grinder to use
    #[arg(long)]
    pub grinder_id: i64,

    /// Grind setting (e.g., 6.0 or 7.5)
    #[arg(long, default_value = "6.0")]
    pub grind_setting: f64,

    /// ID of the brewer to use
    #[arg(long)]
    pub brewer_id: i64,

    /// ID of the filter paper to use (optional)
    #[arg(long)]
    pub filter_paper_id: Option<i64>,

    /// Volume of water in ml
    #[arg(long, default_value = "250")]
    pub water_volume: i32,

    /// Water temperature in Celsius
    #[arg(long, default_value = "91.0")]
    pub water_temp: f64,
}

pub async fn add_brew(client: &BrewlogClient, command: AddBrewCommand) -> Result<()> {
    let brew = client
        .brews()
        .create(
            BagId::new(command.bag_id),
            command.coffee_weight,
            GearId::new(command.grinder_id),
            command.grind_setting,
            GearId::new(command.brewer_id),
            command.filter_paper_id.map(GearId::new),
            command.water_volume,
            command.water_temp,
        )
        .await?;
    print_json(&brew)
}

#[derive(Debug, Args)]
pub struct ListBrewsCommand {
    /// Filter by bag ID
    #[arg(long)]
    pub bag_id: Option<i64>,
}

pub async fn list_brews(client: &BrewlogClient, command: ListBrewsCommand) -> Result<()> {
    let brews = client.brews().list(command.bag_id.map(BagId::new)).await?;
    print_json(&brews)
}

define_get_command!(GetBrewCommand, get_brew, BrewId, brews);
define_delete_command!(DeleteBrewCommand, delete_brew, BrewId, brews, "brew");
