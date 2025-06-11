use rsky_pds::{build_app, run_server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Set up tracing
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    
    // Build the application
    let (app, addr) = build_app(None).await;
    
    // Start the server
    tracing::info!("Starting server at {}", addr);
    run_server(app, addr).await
}
