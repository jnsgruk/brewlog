use anyhow::Result;
use clap::Args;
use serde_json::json;

use super::print_json;
use crate::domain::ids::{BagId, RoastId};
use crate::infrastructure::client::BrewlogClient;

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

#[derive(Debug, Args)]
pub struct GetBagCommand {
    #[arg(long)]
    pub id: i64,
}

pub async fn get_bag(client: &BrewlogClient, command: GetBagCommand) -> Result<()> {
    let bag = client.bags().get(BagId::new(command.id)).await?;
    print_json(&bag)
}

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

#[derive(Debug, Args)]
pub struct DeleteBagCommand {
    #[arg(long)]
    pub id: i64,
}

pub async fn delete_bag(client: &BrewlogClient, command: DeleteBagCommand) -> Result<()> {
    let id = BagId::new(command.id);
    client.bags().delete(id).await?;
    let response = json!({
        "status": "deleted",
        "resource": "bag",
        "id": id.into_inner(),
    });
    print_json(&response)
}
