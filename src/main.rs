use anyhow::Result;
use brewlog::application::{ServerConfig, serve};
use brewlog::infrastructure::client::BrewlogClient;
use brewlog::presentation::cli::{Cli, Commands, ServeCommand, roasters, roasts, tokens};
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
        command => {
            let client = BrewlogClient::from_base_url(&cli.api_url)?;
            match command {
                // Tokens
                Commands::CreateToken(cmd) => tokens::create_token(&client, cmd).await,
                Commands::ListTokens => tokens::list_tokens(&client).await,
                Commands::RevokeToken(cmd) => tokens::revoke_token(&client, cmd).await,

                // Roasters
                Commands::AddRoaster(cmd) => roasters::add_roaster(&client, cmd).await,
                Commands::ListRoasters => roasters::list_roasters(&client).await,
                Commands::GetRoaster(cmd) => roasters::get_roaster(&client, cmd).await,
                Commands::UpdateRoaster(cmd) => roasters::update_roaster(&client, cmd).await,
                Commands::DeleteRoaster(cmd) => roasters::delete_roaster(&client, cmd).await,

                // Roasts
                Commands::AddRoast(cmd) => roasts::add_roast(&client, cmd).await,
                Commands::ListRoasts(cmd) => roasts::list_roasts(&client, cmd).await,
                Commands::GetRoast(cmd) => roasts::get_roast(&client, cmd).await,
                Commands::DeleteRoast(cmd) => roasts::delete_roast(&client, cmd).await,
                Commands::Serve(_) => unreachable!("serve command handled earlier"),
            }
        }
    }
}

async fn run_server(command: ServeCommand) -> Result<()> {
    let config = ServerConfig {
        bind_address: command.bind_address,
        database_url: command.database_url,
        admin_password: command.admin_password,
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
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}
