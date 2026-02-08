use std::net::SocketAddr;

use anyhow::{Context, Result, anyhow};
use clap::{Args, Subcommand};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use super::print_json;
use crate::domain::ids::TokenId;
use crate::infrastructure::auth::generate_session_token;
use crate::infrastructure::client::BrewlogClient;

#[derive(Debug, Subcommand)]
pub enum TokenCommands {
    /// Create a new API token (opens browser for passkey authentication)
    Create(CreateTokenCommand),
    /// List all tokens
    List,
    /// Revoke a token
    Revoke(RevokeTokenCommand),
}

pub async fn run(client: &BrewlogClient, cmd: TokenCommands) -> Result<()> {
    match cmd {
        TokenCommands::Create(c) => create_token(client, c).await,
        TokenCommands::List => list_tokens(client).await,
        TokenCommands::Revoke(c) => revoke_token(client, c).await,
    }
}

#[derive(Debug, Args)]
pub struct CreateTokenCommand {
    /// A descriptive name for this token
    #[arg(long)]
    pub name: String,
}

#[derive(Debug, Args)]
pub struct RevokeTokenCommand {
    /// The ID of the token to revoke
    #[arg(long)]
    pub id: TokenId,
}

pub async fn create_token(client: &BrewlogClient, cmd: CreateTokenCommand) -> Result<()> {
    let state = generate_session_token();

    // Start a local server on a random port to receive the callback
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .context("failed to bind local callback server")?;
    let local_addr = listener
        .local_addr()
        .context("failed to get local callback address")?;

    let callback_url = format!("http://127.0.0.1:{}/callback", local_addr.port());

    // Build the browser URL
    let mut server_url = client
        .endpoint("login")
        .context("failed to build login URL")?;
    server_url
        .query_pairs_mut()
        .append_pair("cli_callback", &callback_url)
        .append_pair("state", &state)
        .append_pair("token_name", &cmd.name);

    println!("Opening browser for authentication...");
    println!("If the browser doesn't open, visit this URL:");
    println!("\n  {server_url}\n");

    // Open the browser
    if let Err(err) = open::that(server_url.as_str()) {
        eprintln!("Warning: failed to open browser: {err}");
    }

    // Wait for the callback
    let (tx, rx) = oneshot::channel::<String>();
    let expected_state = state.clone();

    let server = tokio::spawn(run_callback_server(
        listener,
        local_addr,
        expected_state,
        tx,
    ));

    // Wait for the token with a timeout
    let token = tokio::select! {
        result = rx => {
            result.context("callback server closed without receiving a token")?
        }
        () = tokio::time::sleep(std::time::Duration::from_secs(120)) => {
            return Err(anyhow!("timed out waiting for browser authentication (2 minutes)"));
        }
    };

    // Clean up the server task
    server.abort();

    println!("Token created successfully!");
    println!("Token Name: {}", cmd.name);
    println!("\nSave this token securely - it will not be shown again:");
    println!("\n{token}");
    println!("\nExport it as an environment variable:");
    println!("  export BREWLOG_TOKEN={token}");

    Ok(())
}

async fn run_callback_server(
    listener: TcpListener,
    _addr: SocketAddr,
    expected_state: String,
    tx: oneshot::Sender<String>,
) {
    use axum::extract::Query;
    use axum::response::Html;
    use axum::routing::get;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct CallbackQuery {
        token: Option<String>,
        state: Option<String>,
    }

    let tx = std::sync::Arc::new(tokio::sync::Mutex::new(Some(tx)));
    let state_clone = expected_state.clone();

    let app = axum::Router::new().route(
        "/callback",
        get(move |Query(query): Query<CallbackQuery>| {
            let tx = tx.clone();
            let expected = state_clone.clone();
            async move {
                let Some(token) = query.token else {
                    return Html(
                        "<html><body><h1>Error</h1><p>No token received.</p></body></html>"
                            .to_string(),
                    );
                };

                let Some(received_state) = query.state else {
                    return Html(
                        "<html><body><h1>Error</h1><p>No state parameter.</p></body></html>"
                            .to_string(),
                    );
                };

                if received_state != expected {
                    return Html(
                        "<html><body><h1>Error</h1><p>State mismatch - possible CSRF attack.</p></body></html>"
                            .to_string(),
                    );
                }

                if let Some(sender) = tx.lock().await.take() {
                    let _ = sender.send(token);
                }

                Html("<html><body><h1>Authenticated</h1><p>This window can be closed. Return to the terminal.</p></body></html>".to_string())
            }
        }),
    );

    let _ = axum::serve(listener, app).await;
}

pub async fn list_tokens(client: &BrewlogClient) -> Result<()> {
    let tokens = client.tokens().list().await?;
    print_json(&tokens)
}

pub async fn revoke_token(client: &BrewlogClient, cmd: RevokeTokenCommand) -> Result<()> {
    let token = client.tokens().revoke(cmd.id).await?;
    println!("Token revoked successfully");
    print_json(&token)
}
