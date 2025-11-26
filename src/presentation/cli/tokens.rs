use anyhow::{Context, Result};
use clap::Args;
use std::io::{self, Write};

use super::print_json;
use crate::domain::ids::TokenId;
use crate::infrastructure::client::BrewlogClient;

#[derive(Debug, Args)]
pub struct CreateTokenCommand {
    /// A descriptive name for this token
    #[arg(long)]
    pub name: String,

    /// The username to authenticate with
    #[arg(long)]
    pub username: Option<String>,

    /// The password to authenticate with
    #[arg(long)]
    pub password: Option<String>,
}

#[derive(Debug, Args)]
pub struct RevokeTokenCommand {
    /// The ID of the token to revoke
    #[arg(long)]
    pub id: TokenId,
}

pub async fn create_token(client: &BrewlogClient, cmd: CreateTokenCommand) -> Result<()> {
    let username = if let Some(u) = cmd.username {
        u
    } else {
        // Prompt for username
        print!("Username: ");
        io::stdout().flush()?;
        let mut username = String::new();
        io::stdin().read_line(&mut username)?;
        username.trim().to_string()
    };

    let password = if let Some(p) = cmd.password {
        p
    } else {
        // Prompt for password (without echo)
        rpassword::prompt_password("Password: ").context("failed to read password")?
    };

    // Create the token
    let token_response = client
        .tokens()
        .create(&username, &password, &cmd.name)
        .await?;

    println!("\nToken created successfully!");
    println!("Token ID: {}", token_response.id);
    println!("Token Name: {}", token_response.name);
    println!("\n⚠️  Save this token securely - it will not be shown again:");
    println!("\n{}", token_response.token);
    println!("\nExport it in your environment:");
    println!("  export BREWLOG_TOKEN={}", token_response.token);

    Ok(())
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
