use std::net::SocketAddr;

use axum::Router;
use clap::Parser;

use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::{debug, error, info, Level};
use tracing_subscriber::EnvFilter;

use tinysse::{cli::Cli, state::AppState, web};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(cli.log_level.as_str()))
        .init();

    debug!("{cli:?}");

    if let Err(e) = try_main(&cli).await {
        error!("{e}");
        std::process::exit(1);
    }
}

async fn try_main(cli: &Cli) -> anyhow::Result<()> {
    let state = AppState::from_cli(cli).await?;
    debug!("{state:?}");

    let router = Router::new()
        .merge(web::router(&state))
        .layer(
            ServiceBuilder::new().layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().include_headers(true))
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .latency_unit(LatencyUnit::Millis),
                    ),
            ),
        )
        .with_state(state)
        .into_make_service_with_connect_info::<SocketAddr>();

    let listener = TcpListener::bind(&cli.listen).await?;
    let local_addr = listener.local_addr()?;
    info!("Listening on {local_addr}");

    axum::serve(listener, router).await?;

    Ok(())
}
