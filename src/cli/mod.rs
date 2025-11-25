pub mod roasters;
pub mod roasts;

use std::net::SocketAddr;

use clap::{Args, Parser, Subcommand};
use roasters::{AddRoasterCommand, DeleteRoasterCommand, GetRoasterCommand, UpdateRoasterCommand};
use roasts::{AddRoastCommand, DeleteRoastCommand, GetRoastCommand, ListRoastsCommand};

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
}

pub(crate) fn print_json<T>(value: &T) -> anyhow::Result<()>
where
    T: serde::Serialize,
{
    let rendered = serde_json::to_string_pretty(value)?;
    println!("{rendered}");
    Ok(())
}
