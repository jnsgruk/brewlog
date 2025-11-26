use anyhow::Result;
use clap::Args;
use serde_json::json;

use super::print_json;
use crate::client::BrewlogClient;
use crate::domain::ids::RoasterId;
use crate::domain::roasters::{NewRoaster, UpdateRoaster};

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
    #[arg(long)]
    pub notes: Option<String>,
}

pub async fn add_roaster(client: &BrewlogClient, command: AddRoasterCommand) -> Result<()> {
    let payload = NewRoaster {
        name: command.name,
        country: command.country,
        city: command.city,
        homepage: command.homepage,
        notes: command.notes,
    };

    let roaster = client.roasters().create(&payload).await?;
    print_json(&roaster)
}

pub async fn list_roasters(client: &BrewlogClient) -> Result<()> {
    let roasters = client.roasters().list().await?;
    print_json(&roasters)
}

#[derive(Debug, Args)]
pub struct GetRoasterCommand {
    #[arg(long)]
    pub id: i64,
}

pub async fn get_roaster(client: &BrewlogClient, command: GetRoasterCommand) -> Result<()> {
    let roaster = client.roasters().get(RoasterId::new(command.id)).await?;
    print_json(&roaster)
}

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
    #[arg(long)]
    pub notes: Option<String>,
}

pub async fn update_roaster(client: &BrewlogClient, command: UpdateRoasterCommand) -> Result<()> {
    let payload = UpdateRoaster {
        name: command.name,
        country: command.country,
        city: command.city,
        homepage: command.homepage,
        notes: command.notes,
    };

    let roaster = client
        .roasters()
        .update(RoasterId::new(command.id), &payload)
        .await?;
    print_json(&roaster)
}

#[derive(Debug, Args)]
pub struct DeleteRoasterCommand {
    #[arg(long)]
    pub id: i64,
}

pub async fn delete_roaster(client: &BrewlogClient, command: DeleteRoasterCommand) -> Result<()> {
    let id = RoasterId::new(command.id);
    client.roasters().delete(id).await?;
    let response = json!({
        "status": "deleted",
        "resource": "roaster",
        "id": id.into_inner(),
    });
    print_json(&response)
}
