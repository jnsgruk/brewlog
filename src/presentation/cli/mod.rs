pub mod bags;
pub mod gear;
mod macros;
pub mod roasters;
pub mod roasts;
pub mod tokens;

use std::net::SocketAddr;

use bags::{AddBagCommand, DeleteBagCommand, GetBagCommand, ListBagsCommand, UpdateBagCommand};
use clap::{Args, Parser, Subcommand};
use gear::{
    AddGearCommand, DeleteGearCommand, GetGearCommand, ListGearCommand, UpdateGearCommand,
};
use roasters::{AddRoasterCommand, DeleteRoasterCommand, GetRoasterCommand, UpdateRoasterCommand};
use roasts::{AddRoastCommand, DeleteRoastCommand, GetRoastCommand, ListRoastsCommand};
use tokens::{CreateTokenCommand, RevokeTokenCommand};

#[derive(Debug, Parser)]
#[command(author, version, about = "Track coffee roasts, brews, and cups", long_about = None)]
pub struct Cli {
    #[arg(
        long,
        global = true,
        env = "BREWLOG_URL",
        default_value = "http://127.0.0.1:3000"
    )]
    pub api_url: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(name = "serve")]
    Serve(ServeCommand),

    // Tokens
    #[command(name = "create-token")]
    CreateToken(CreateTokenCommand),
    #[command(name = "list-tokens")]
    ListTokens,
    #[command(name = "revoke-token")]
    RevokeToken(RevokeTokenCommand),

    // Roasters
    #[command(name = "add-roaster")]
    AddRoaster(AddRoasterCommand),
    #[command(name = "list-roasters")]
    ListRoasters,
    #[command(name = "get-roaster")]
    GetRoaster(GetRoasterCommand),
    #[command(name = "update-roaster")]
    UpdateRoaster(UpdateRoasterCommand),
    #[command(name = "delete-roaster")]
    DeleteRoaster(DeleteRoasterCommand),

    // Roasts
    #[command(name = "add-roast")]
    AddRoast(AddRoastCommand),
    #[command(name = "list-roasts")]
    ListRoasts(ListRoastsCommand),
    #[command(name = "get-roast")]
    GetRoast(GetRoastCommand),
    #[command(name = "delete-roast")]
    DeleteRoast(DeleteRoastCommand),

    // Bags
    #[command(name = "add-bag")]
    AddBag(AddBagCommand),
    #[command(name = "list-bags")]
    ListBags(ListBagsCommand),
    #[command(name = "get-bag")]
    GetBag(GetBagCommand),
    #[command(name = "update-bag")]
    UpdateBag(UpdateBagCommand),
    #[command(name = "delete-bag")]
    DeleteBag(DeleteBagCommand),

    // Gear
    #[command(name = "add-gear")]
    AddGear(AddGearCommand),
    #[command(name = "list-gear")]
    ListGear(ListGearCommand),
    #[command(name = "get-gear")]
    GetGear(GetGearCommand),
    #[command(name = "update-gear")]
    UpdateGear(UpdateGearCommand),
    #[command(name = "delete-gear")]
    DeleteGear(DeleteGearCommand),
}

#[derive(Debug, Args)]
pub struct ServeCommand {
    #[arg(
        long,
        env = "BREWLOG_DATABASE_URL",
        default_value = "sqlite://brewlog.db"
    )]
    pub database_url: String,

    #[arg(long, env = "BREWLOG_BIND_ADDRESS", default_value = "127.0.0.1:3000")]
    pub bind_address: SocketAddr,

    #[arg(long, env = "BREWLOG_ADMIN_PASSWORD")]
    pub admin_password: Option<String>,

    #[arg(long, env = "BREWLOG_ADMIN_USERNAME")]
    pub admin_username: Option<String>,
}

pub(crate) fn print_json<T>(value: &T) -> anyhow::Result<()>
where
    T: serde::Serialize,
{
    let rendered = serde_json::to_string_pretty(value)?;
    println!("{rendered}");
    Ok(())
}
