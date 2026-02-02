use anyhow::Result;
use clap::Args;

use super::macros::{define_delete_command, define_get_command};
use super::print_json;
use crate::domain::ids::RoasterId;
use crate::domain::roasters::{NewRoaster, UpdateRoaster};
use crate::infrastructure::client::BrewlogClient;

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

define_delete_command!(DeleteRoasterCommand, delete_roaster, RoasterId, roasters, "roaster");
