use anyhow::Result;
use clap::{Args, Subcommand};

use super::macros::{define_delete_command, define_get_command};
use super::parse_created_at;
use super::print_json;
use crate::domain::brews::QuickNote;
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

    /// Quick notes (comma-separated: good,too-fast,too-slow,too-hot,under-extracted,over-extracted)
    #[arg(long, value_delimiter = ',')]
    pub quick_notes: Vec<String>,

    /// Brew time in seconds (e.g., 150 for 2:30)
    #[arg(long)]
    pub brew_time: Option<i32>,

    /// Override creation timestamp (e.g. 2025-08-05T10:00:00Z or 2025-08-05)
    #[arg(long)]
    pub created_at: Option<String>,
}

pub async fn add_brew(client: &BrewlogClient, command: AddBrewCommand) -> Result<()> {
    let quick_notes: Vec<QuickNote> = command
        .quick_notes
        .iter()
        .filter_map(|s| QuickNote::from_str_value(s))
        .collect();
    let created_at = command
        .created_at
        .map(|s| parse_created_at(&s))
        .transpose()?;

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
            quick_notes,
            command.brew_time,
            created_at,
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
