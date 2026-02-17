use arca::{transport, PadronA13Service};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Test Dummy Padrón A13 ===\n");

    let http = transport::HttpClient::new()?;
    
    println!("Testing homologación...");
    let padron = PadronA13Service::testing();
    match padron.dummy(&http).await {
        Ok(d) => println!("  App: {}, Auth: {}, DB: {}", d.app_server, d.auth_server, d.db_server),
        Err(e) => println!("  Error: {}", e),
    }

    println!("\nTesting producción...");
    let padron = PadronA13Service::production();
    match padron.dummy(&http).await {
        Ok(d) => println!("  App: {}, Auth: {}, DB: {}", d.app_server, d.auth_server, d.db_server),
        Err(e) => println!("  Error: {}", e),
    }

    Ok(())
}
