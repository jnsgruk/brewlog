pub mod bags;
pub mod brews;
pub mod gear;
mod macros;
pub mod roasters;
pub mod roasts;
pub mod tokens;

use std::net::SocketAddr;

use bags::BagCommands;
use brews::BrewCommands;
use clap::{Args, Parser, Subcommand};
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

    /// Manage API tokens
    Token {
        #[command(subcommand)]
        command: TokenCommands,
    },
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
