use anyhow::Result;
use clap::Args;
use serde_json::json;

use super::print_json;
use crate::client::BrewlogClient;
use crate::domain::roasts::NewRoast;

#[derive(Debug, Args)]
pub struct AddRoastCommand {
    #[arg(long)]
    pub roaster_id: String,
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub origin: Option<String>,
    #[arg(long)]
    pub region: Option<String>,
    #[arg(long)]
    pub producer: Option<String>,
    #[arg(long)]
    pub process: Option<String>,
    #[arg(long = "tasting-notes")]
    pub tasting_notes: Vec<String>,
}

pub async fn add_roast(client: &BrewlogClient, command: AddRoastCommand) -> Result<()> {
    let payload = NewRoast {
        roaster_id: command.roaster_id,
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
    pub roaster_id: Option<String>,
}

pub async fn list_roasts(client: &BrewlogClient, command: ListRoastsCommand) -> Result<()> {
    let roasts = client.roasts().list(command.roaster_id.as_deref()).await?;
    print_json(&roasts)
}

#[derive(Debug, Args)]
pub struct GetRoastCommand {
    #[arg(long)]
    pub id: String,
}

pub async fn get_roast(client: &BrewlogClient, command: GetRoastCommand) -> Result<()> {
    let roast = client.roasts().get(&command.id).await?;
    print_json(&roast)
}

#[derive(Debug, Args)]
pub struct DeleteRoastCommand {
    #[arg(long)]
    pub id: String,
}

pub async fn delete_roast(client: &BrewlogClient, command: DeleteRoastCommand) -> Result<()> {
    let id = command.id;
    client.roasts().delete(&id).await?;
    let response = json!({
        "status": "deleted",
        "resource": "roast",
        "id": id,
    });
    print_json(&response)
}
