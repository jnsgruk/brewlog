pub mod backup;
pub mod bags;
pub mod brews;
pub mod cafes;
pub mod cups;
pub mod gear;
mod macros;
pub mod roasters;
pub mod roasts;
pub mod tokens;

use std::net::SocketAddr;

use backup::{BackupCommand, RestoreCommand};
use bags::BagCommands;
use brews::BrewCommands;
use cafes::CafeCommands;
use clap::{Args, Parser, Subcommand};
use cups::CupCommands;
use gear::GearCommands;
use roasters::RoasterCommands;
use roasts::RoastCommands;
use tokens::TokenCommands;

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
    /// Run the HTTP server
    Serve(ServeCommand),

    /// Manage roasters
    Roaster {
        #[command(subcommand)]
        command: RoasterCommands,
    },

    /// Manage roasts
    Roast {
        #[command(subcommand)]
        command: RoastCommands,
    },

    /// Manage bags
    Bag {
        #[command(subcommand)]
        command: BagCommands,
    },

    /// Manage gear
    Gear {
        #[command(subcommand)]
        command: GearCommands,
    },

    /// Manage brews
    Brew {
        #[command(subcommand)]
        command: BrewCommands,
    },

    /// Manage cafes
    Cafe {
        #[command(subcommand)]
        command: CafeCommands,
    },

    /// Manage cups
    Cup {
        #[command(subcommand)]
        command: CupCommands,
    },

    /// Manage API tokens
    Token {
        #[command(subcommand)]
        command: TokenCommands,
    },

    /// Back up all coffee data to JSON (stdout)
    Backup(BackupCommand),

    /// Restore coffee data from a JSON backup file
    Restore(RestoreCommand),
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

    #[arg(long, env = "BREWLOG_OPENROUTER_API_KEY")]
    pub openrouter_api_key: Option<String>,

    #[arg(
        long,
        env = "BREWLOG_OPENROUTER_MODEL",
        default_value = "openrouter/free"
    )]
    pub openrouter_model: String,
}

pub(crate) fn print_json<T>(value: &T) -> anyhow::Result<()>
where
    T: serde::Serialize,
{
    let rendered = serde_json::to_string_pretty(value)?;
    println!("{rendered}");
    Ok(())
}
