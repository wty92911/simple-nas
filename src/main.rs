use anyhow::Result;
use std::{net::SocketAddr, sync::Arc};
use tower::ServiceBuilder;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::{Level, info};

// Import necessary components
use clap::Parser;
use simple_nas::config::AppConfig;
use simple_nas::handlers::AppState;
use simple_nas::routes::create_router;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// path to the config file
    #[arg(short, long, default_value = "./fixtures/configs/app_config.yml")]
    config_path: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(Level::INFO)
        .init();

    info!("🚀 Starting Simple Home NAS server...");

    let args = Args::parse();
    // Print the parsed args
    info!("🔍 Parsed arguments: {:?}", args);

    // Load application configuration from environment
    let app_config = AppConfig::from_yml(args.config_path)?;

    info!("✅ Configuration loaded successfully");

    // Create application state
    let app_state = Arc::new(AppState::new(&app_config).await?);

    info!("🔐 Security infrastructure initialized");

    let service = ServiceBuilder::new().layer(
        TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)),
    );
    // TODO: add rate limit and concurrency limit

    // Build our application with routes
    let app = create_router(app_state).layer(service);

    let addr = SocketAddr::from(([127, 0, 0, 1], app_config.port));
    // Start server
    info!("🌐 Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
