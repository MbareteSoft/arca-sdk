use clap::Parser;
use anyhow::Result;
use std::fs;

use arca::{ArcaClient, ArcaEnvironment};

#[derive(Parser)]
struct Args {
    #[arg(long)]
    env: String,
    #[arg(long)]
    cert_pem: String,
    #[arg(long)]
    key_pem: String,
    #[arg(long, default_value = "wsfe")]
    service: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let env = ArcaEnvironment::from_str(&args.env)
        .ok_or_else(|| anyhow::anyhow!("Invalid environment: {}", args.env))?;

    let cert = fs::read(&args.cert_pem)?;
    let key = fs::read(&args.key_pem)?;

    let client = ArcaClient::builder()
        .environment(env)
        .credentials(cert, key)
        .build()?;

    client.login(&args.service).await?;

    println!("Login successful for service: {}", args.service);
    println!("Environment: {:?}", client.environment());
    println!("Client ready. To test specific methods, add code to main.rs");

    Ok(())
}
