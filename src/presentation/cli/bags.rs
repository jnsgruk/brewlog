use anyhow::Result;
use clap::{Args, Subcommand};

use super::macros::{define_delete_command, define_get_command};
use super::print_json;
use crate::domain::ids::{BagId, RoastId};
use crate::infrastructure::client::BrewlogClient;

#[derive(Debug, Subcommand)]
pub enum BagCommands {
    /// Add a new bag
    Add(AddBagCommand),
    /// List all bags
    List(ListBagsCommand),
    /// Get a bag by ID
    Get(GetBagCommand),
    /// Update a bag
    Update(UpdateBagCommand),
    /// Delete a bag
    Delete(DeleteBagCommand),
}

pub async fn run(client: &BrewlogClient, cmd: BagCommands) -> Result<()> {
    match cmd {
        BagCommands::Add(c) => add_bag(client, c).await,
        BagCommands::List(c) => list_bags(client, c).await,
        BagCommands::Get(c) => get_bag(client, c).await,
        BagCommands::Update(c) => update_bag(client, c).await,
        BagCommands::Delete(c) => delete_bag(client, c).await,
    }
}

#[derive(Debug, Args)]
pub struct AddBagCommand {
    #[arg(long)]
    pub roast_id: i64,
    #[arg(long)]
    pub roast_date: Option<String>,
    #[arg(long)]
    pub amount: f64,
}

pub async fn add_bag(client: &BrewlogClient, command: AddBagCommand) -> Result<()> {
    let roast_date = command
        .roast_date
        .map(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d"))
        .transpose()?;
    let bag = client
        .bags()
        .create(RoastId::new(command.roast_id), roast_date, command.amount)
        .await?;
    print_json(&bag)
}

#[derive(Debug, Args)]
pub struct ListBagsCommand {
    #[arg(long)]
    pub roast_id: Option<i64>,
}

pub async fn list_bags(client: &BrewlogClient, command: ListBagsCommand) -> Result<()> {
    let bags = client
        .bags()
        .list(command.roast_id.map(RoastId::new))
        .await?;
    print_json(&bags)
}

define_get_command!(GetBagCommand, get_bag, BagId, bags);

#[derive(Debug, Args)]
pub struct UpdateBagCommand {
    #[arg(long)]
    pub id: i64,
    #[arg(long)]
    pub remaining: Option<f64>,
    #[arg(long)]
    pub closed: Option<bool>,
    #[arg(long)]
    pub finished_at: Option<String>,
}

pub async fn update_bag(client: &BrewlogClient, command: UpdateBagCommand) -> Result<()> {
    let finished_at = command
        .finished_at
        .map(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d"))
        .transpose()?;

    let bag = client
        .bags()
        .update(
            BagId::new(command.id),
            command.remaining,
            command.closed,
            finished_at,
        )
        .await?;
    print_json(&bag)
}

define_delete_command!(DeleteBagCommand, delete_bag, BagId, bags, "bag");
