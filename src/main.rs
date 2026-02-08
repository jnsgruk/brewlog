use anyhow::Result;
use brewlog::application::{ServerConfig, serve};
use brewlog::infrastructure::backup::BackupData;
use brewlog::infrastructure::client::BrewlogClient;
use brewlog::presentation::cli::{
    Cli, Commands, ServeCommand, bags, brews, cafes, cups, gear, roasters, roasts, tokens,
};
use clap::Parser;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if present (before clap parses env vars)
    let _ = dotenvy::dotenv();

    init_tracing();

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
        Commands::Cafe { command } => {
            let client = BrewlogClient::from_base_url(&cli.api_url)?;
            cafes::run(&client, command).await
        }
        Commands::Cup { command } => {
            let client = BrewlogClient::from_base_url(&cli.api_url)?;
            cups::run(&client, command).await
        }
        Commands::Token { command } => {
            let client = BrewlogClient::from_base_url(&cli.api_url)?;
            tokens::run(&client, command).await
        }
        Commands::Backup(_cmd) => {
            let client = BrewlogClient::from_base_url(&cli.api_url)?;
            let data = client.backup().export().await?;
            let json = serde_json::to_string_pretty(&data)?;
            println!("{json}");
            Ok(())
        }
        Commands::Restore(cmd) => {
            let contents = std::fs::read_to_string(&cmd.file)?;
            let data: BackupData = serde_json::from_str(&contents)?;
            let client = BrewlogClient::from_base_url(&cli.api_url)?;
            client.backup().restore(&data).await?;
            eprintln!("Restore complete.");
            Ok(())
        }
    }
}

async fn run_server(command: ServeCommand) -> Result<()> {
    let rp_id = command.rp_id.ok_or_else(|| {
        anyhow::anyhow!(
            "BREWLOG_RP_ID is required. Set this to the domain where the app is hosted \
             (e.g. 'brewlog.example.com' or 'localhost')."
        )
    })?;

    let rp_origin = command.rp_origin.ok_or_else(|| {
        anyhow::anyhow!(
            "BREWLOG_RP_ORIGIN is required. Set this to the full origin URL \
             (e.g. 'https://brewlog.example.com' or 'http://localhost:3000')."
        )
    })?;

    let openrouter_api_key = command.openrouter_api_key.ok_or_else(|| {
        anyhow::anyhow!(
            "BREWLOG_OPENROUTER_API_KEY is required. Set this environment variable \
             to an OpenRouter API key for AI-powered extraction features."
        )
    })?;

    let foursquare_api_key = command.foursquare_api_key.ok_or_else(|| {
        anyhow::anyhow!(
            "BREWLOG_FOURSQUARE_API_KEY is required. Set this environment variable \
             to a Foursquare API key for nearby cafe search."
        )
    })?;

    let config = ServerConfig {
        bind_address: command.bind_address,
        database_url: command.database_url,
        rp_id,
        rp_origin,
        openrouter_api_key,
        openrouter_model: command.openrouter_model,
        foursquare_api_key,
    };

    serve(config).await
}

#[allow(clippy::expect_used)] // Startup: panicking is appropriate if logging cannot be initialized
fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let use_json = std::env::var("RUST_LOG_FORMAT").is_ok_and(|v| v.eq_ignore_ascii_case("json"));

    let registry = tracing_subscriber::registry().with(env_filter);

    if use_json {
        registry
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        registry
            .with(tracing_subscriber::fmt::layer().compact())
            .init();
    }
}
