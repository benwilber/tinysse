use std::net::SocketAddr;

use axum::Router;
use clap::Parser;

use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    LatencyUnit, cors,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{Level, debug, error, info};
use tracing_subscriber::EnvFilter;

use tinysse::{cli::Cli, state::AppState, web};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(cli.log_level.as_str()))
        .init();

    debug!("cli={cli:?}");

    if let Err(e) = try_main(&cli).await {
        error!("{e}");
        std::process::exit(1);
    }
}

async fn try_main(cli: &Cli) -> anyhow::Result<()> {
    let state = AppState::from_cli(cli).await?;
    debug!("state={state:?}");

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
        .layer(
            ServiceBuilder::new().layer(
                cors::CorsLayer::new()
                    .allow_origin(cli.cors_allow_origin.clone())
                    .allow_methods(cli.cors_allow_methods.clone())
                    .allow_headers(cli.cors_allow_headers.clone())
                    .allow_credentials(cli.cors_allow_credentials)
                    .max_age(cli.cors_max_age),
            ),
        )
        .with_state(state.clone())
        .into_make_service_with_connect_info::<SocketAddr>();

    let listener = TcpListener::bind(&cli.listen).await?;
    let local_addr = listener.local_addr()?;
    info!("Listening on {local_addr}");

    state.script.on_startup(cli).await?;

    tokio::select! {
        _ = async {
            // Run the script tick loop
            let mut count = 0;

            loop {
                count += 1;
                if let Err(e) = state.script.on_tick(count).await {
                    // Don't crash the server if the script tick fails.
                    // Just log it.
                    error!("{e}");
                }

                tokio::time::sleep(cli.script_tick).await;
            }
        } => {},

        result = axum::serve(listener, router) => {
            result?;
        }
    }

    Ok(())
}
