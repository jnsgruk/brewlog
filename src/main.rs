use anyhow::Result;
use brewlog::application::{ServerConfig, serve};
use brewlog::infrastructure::backup::{BackupData, BackupService};
use brewlog::infrastructure::client::BrewlogClient;
use brewlog::infrastructure::database::Database;
use brewlog::presentation::cli::{
    Cli, Commands, ServeCommand, bags, brews, gear, roasters, roasts, tokens,
};
use clap::Parser;

use tracing::{Subscriber, subscriber::set_global_default};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt};

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = get_subscriber("brewlog".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve(cmd) => run_server(cmd).await,
        Commands::Roaster { command } => {
            let client = BrewlogClient::from_base_url(&cli.api_url)?;
            roasters::run(&client, command).await
        }
        Commands::Roast { command } => {
            let client = BrewlogClient::from_base_url(&cli.api_url)?;
            roasts::run(&client, command).await
        }
        Commands::Bag { command } => {
            let client = BrewlogClient::from_base_url(&cli.api_url)?;
            bags::run(&client, command).await
        }
        Commands::Gear { command } => {
            let client = BrewlogClient::from_base_url(&cli.api_url)?;
            gear::run(&client, command).await
        }
        Commands::Brew { command } => {
            let client = BrewlogClient::from_base_url(&cli.api_url)?;
            brews::run(&client, command).await
        }
        Commands::Token { command } => {
            let client = BrewlogClient::from_base_url(&cli.api_url)?;
            tokens::run(&client, command).await
        }
        Commands::Backup(cmd) => {
            let database = Database::connect(&cmd.database_url).await?;
            let service = BackupService::new(database.clone_pool());
            let data = service.export().await?;
            let json = serde_json::to_string_pretty(&data)?;
            println!("{json}");
            Ok(())
        }
        Commands::Restore(cmd) => {
            let contents = std::fs::read_to_string(&cmd.file)?;
            let data: BackupData = serde_json::from_str(&contents)?;
            let database = Database::connect(&cmd.database_url).await?;
            let service = BackupService::new(database.clone_pool());
            service.restore(data).await?;
            eprintln!("Restore complete.");
            Ok(())
        }
    }
}

async fn run_server(command: ServeCommand) -> Result<()> {
    let config = ServerConfig {
        bind_address: command.bind_address,
        database_url: command.database_url,
        admin_password: command.admin_password,
        admin_username: command.admin_username,
    };

    serve(config).await
}

pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(name, sink);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Register a subscriber as global default to process span data.
///
/// This should only be called once!
#[allow(clippy::expect_used)] // Startup: panicking is appropriate if logging cannot be initialized
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}
