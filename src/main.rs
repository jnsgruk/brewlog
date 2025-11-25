use anyhow::Result;
use brewlog::application::{ServerConfig, serve};
use brewlog::cli::{Cli, Commands, ServeCommand, roasters, roasts, tokens};
use brewlog::client::BrewlogClient;
use clap::Parser;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(err) = init_tracing() {
        eprintln!("failed to initialize tracing: {err}");
    }

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

fn init_tracing() -> anyhow::Result<()> {
    let env_filter = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .try_init()
        .map_err(|err| anyhow::anyhow!("failed to initialize tracing: {err}"))
}
