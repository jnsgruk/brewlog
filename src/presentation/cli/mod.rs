pub mod backup;
pub mod bags;
pub mod brews;
pub mod cafes;
pub mod cups;
pub mod gear;
mod macros;
pub mod roasters;
pub mod roasts;
pub mod timeline;
pub mod tokens;

use std::net::SocketAddr;

use chrono::{DateTime, NaiveDate, Utc};

use backup::{BackupCommand, RestoreCommand};
use bags::BagCommands;
use brews::BrewCommands;
use cafes::CafeCommands;
use clap::{Args, Parser, Subcommand};
use cups::CupCommands;
use gear::GearCommands;
use roasters::RoasterCommands;
use roasts::RoastCommands;
use timeline::TimelineCommands;
use tokens::TokenCommands;

#[derive(Debug, Parser)]
#[command(author, version, about = "Track coffee roasts, brews, and cups", long_about = None)]
pub struct Cli {
    #[arg(
        long,
        global = true,
        env = "BREWLOG_URL",
        default_value = "http://localhost:3000"
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

    /// Manage timeline events
    Timeline {
        #[command(subcommand)]
        command: TimelineCommands,
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

    #[arg(long, env = "BREWLOG_RP_ID", default_value = "localhost")]
    pub rp_id: String,

    #[arg(
        long,
        env = "BREWLOG_RP_ORIGIN",
        default_value = "http://localhost:3000"
    )]
    pub rp_origin: String,

    #[arg(long, env = "BREWLOG_INSECURE_COOKIES")]
    pub insecure_cookies: bool,

    #[arg(long, env = "BREWLOG_OPENROUTER_API_KEY")]
    pub openrouter_api_key: Option<String>,

    #[arg(
        long,
        env = "BREWLOG_OPENROUTER_MODEL",
        default_value = "openrouter/free"
    )]
    pub openrouter_model: String,

    #[arg(long, env = "BREWLOG_FOURSQUARE_API_KEY")]
    pub foursquare_api_key: Option<String>,
}

pub fn parse_created_at(value: &str) -> anyhow::Result<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Ok(dt.with_timezone(&Utc));
    }
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return Ok(date.and_time(chrono::NaiveTime::MIN).and_utc());
    }
    anyhow::bail!(
        "invalid date format: expected RFC 3339 (e.g. 2025-08-05T10:00:00Z) or YYYY-MM-DD"
    )
}

pub(crate) fn print_json<T>(value: &T) -> anyhow::Result<()>
where
    T: serde::Serialize,
{
    let rendered = serde_json::to_string_pretty(value)?;
    println!("{rendered}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_created_at_rfc3339() {
        let result = parse_created_at("2025-08-05T10:30:00Z").unwrap();
        assert_eq!(
            result,
            DateTime::parse_from_rfc3339("2025-08-05T10:30:00Z").unwrap()
        );
    }

    #[test]
    fn parse_created_at_yyyy_mm_dd() {
        let result = parse_created_at("2025-08-05").unwrap();
        let expected = NaiveDate::from_ymd_opt(2025, 8, 5)
            .unwrap()
            .and_time(chrono::NaiveTime::MIN)
            .and_utc();
        assert_eq!(result, expected);
    }

    #[test]
    fn parse_created_at_invalid() {
        let result = parse_created_at("not-a-date");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("invalid date format"));
    }

    #[test]
    fn parse_created_at_leap_day() {
        let result = parse_created_at("2024-02-29").unwrap();
        let expected = NaiveDate::from_ymd_opt(2024, 2, 29)
            .unwrap()
            .and_time(chrono::NaiveTime::MIN)
            .and_utc();
        assert_eq!(result, expected);
    }
}
