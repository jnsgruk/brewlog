use anyhow::Result;
use clap::{Args, Subcommand};

use super::macros::{define_delete_command, define_get_command};
use super::print_json;
use crate::domain::cafes::{NewCafe, UpdateCafe};
use crate::domain::ids::CafeId;
use crate::infrastructure::client::BrewlogClient;

#[derive(Debug, Subcommand)]
pub enum CafeCommands {
    /// Add a new cafe
    Add(AddCafeCommand),
    /// List all cafes
    List,
    /// Get a cafe by ID
    Get(GetCafeCommand),
    /// Update a cafe
    Update(UpdateCafeCommand),
    /// Delete a cafe
    Delete(DeleteCafeCommand),
}

pub async fn run(client: &BrewlogClient, cmd: CafeCommands) -> Result<()> {
    match cmd {
        CafeCommands::Add(c) => add_cafe(client, c).await,
        CafeCommands::List => list_cafes(client).await,
        CafeCommands::Get(c) => get_cafe(client, c).await,
        CafeCommands::Update(c) => update_cafe(client, c).await,
        CafeCommands::Delete(c) => delete_cafe(client, c).await,
    }
}

#[derive(Debug, Args)]
pub struct AddCafeCommand {
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub city: String,
    #[arg(long)]
    pub country: String,
    #[arg(long, allow_negative_numbers = true)]
    pub latitude: f64,
    #[arg(long, allow_negative_numbers = true)]
    pub longitude: f64,
    #[arg(long)]
    pub website: Option<String>,
    #[arg(long)]
    pub notes: Option<String>,
}

pub async fn add_cafe(client: &BrewlogClient, command: AddCafeCommand) -> Result<()> {
    let payload = NewCafe {
        name: command.name,
        city: command.city,
        country: command.country,
        latitude: command.latitude,
        longitude: command.longitude,
        website: command.website,
        notes: command.notes,
    };

    let cafe = client.cafes().create(&payload).await?;
    print_json(&cafe)
}

pub async fn list_cafes(client: &BrewlogClient) -> Result<()> {
    let cafes = client.cafes().list().await?;
    print_json(&cafes)
}

define_get_command!(GetCafeCommand, get_cafe, CafeId, cafes);

#[derive(Debug, Args)]
pub struct UpdateCafeCommand {
    #[arg(long)]
    pub id: i64,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub city: Option<String>,
    #[arg(long)]
    pub country: Option<String>,
    #[arg(long, allow_negative_numbers = true)]
    pub latitude: Option<f64>,
    #[arg(long, allow_negative_numbers = true)]
    pub longitude: Option<f64>,
    #[arg(long)]
    pub website: Option<String>,
    #[arg(long)]
    pub notes: Option<String>,
}

pub async fn update_cafe(client: &BrewlogClient, command: UpdateCafeCommand) -> Result<()> {
    let payload = UpdateCafe {
        name: command.name,
        city: command.city,
        country: command.country,
        latitude: command.latitude,
        longitude: command.longitude,
        website: command.website,
        notes: command.notes,
    };

    let cafe = client
        .cafes()
        .update(CafeId::new(command.id), &payload)
        .await?;
    print_json(&cafe)
}

define_delete_command!(DeleteCafeCommand, delete_cafe, CafeId, cafes, "cafe");
