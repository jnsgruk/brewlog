use anyhow::Result;
use clap::Args;

use super::macros::{define_delete_command, define_get_command};
use super::print_json;
use crate::domain::ids::{RoastId, RoasterId};
use crate::domain::roasts::NewRoast;
use crate::infrastructure::client::BrewlogClient;

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
}

pub async fn add_roast(client: &BrewlogClient, command: AddRoastCommand) -> Result<()> {
    let payload = NewRoast {
        roaster_id: RoasterId::new(command.roaster_id),
        name: command.name,
        origin: command.origin,
        region: command.region,
        producer: command.producer,
        tasting_notes: command.tasting_notes,
        process: command.process,
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
define_delete_command!(DeleteRoastCommand, delete_roast, RoastId, roasts, "roast");
