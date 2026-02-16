use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::{
    body::Body,
    extract::Request,
    middleware::{self, Next},
    response::Response,
    routing::get,
};
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use axum::http::header::{X_CONTENT_TYPE_OPTIONS, X_FRAME_OPTIONS};
use axum::http::HeaderValue;
use tower_http::set_header::SetResponseHeaderLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use shared::db::{Database, DbConfig};

mod config;
mod css;
mod error;
mod routes;
mod state;
mod templates;

use config::Config;
use state::AppState;

async fn log_request(request: Request, next: Next) -> Response {
    let ip = request
        .headers()
        .get("CF-Connecting-IP")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path();

    let is_real_browser = user_agent.starts_with("Mozilla/5.0");
    
    let is_static = path.starts_with("/static")
        || path.starts_with("/wasm")
        || path.ends_with(".ico");
    
    let source = request.uri().query()
        .and_then(|q| {
            q.split('&')
                .find(|p| p.starts_with("utm_source="))
                .map(|p| p.trim_start_matches("utm_source="))
        })
        .map(|s| match s {
            "ig" => " :: Instagram",
            "fb" => " :: Facebook",
            "tw" => " :: Twitter",
            "li" => " :: LinkedIn",
            _ => s
        })
        .unwrap_or("");
    
    if is_real_browser && !is_static {
        tracing::info!("Request: {ip} {method} {path}{source}");
    }
    
    next.run(request).await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "portfolio=debug,tower_http=debug".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_timer(tracing_subscriber::fmt::time::UtcTime::new(
                    time::format_description::parse("[day]/[month]/[year] [hour]:[minute]:[second]").unwrap()
                ))
        )
        .init();

    info!("Starting Portfolio Gateway...");

    let config = Config::from_env()?;
    info!(host = %config.host, port = %config.port, "Configuration loaded");

    let db_config = DbConfig::from_env()?;
    let db = Database::connect(&db_config).await?;
    db.init_schema().await?;

    let state = Arc::new(AppState::new(db, config.clone()));

    use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
    use tower_governor::key_extractor::SmartIpKeyExtractor;

    let governor_config = Arc::new(
    GovernorConfigBuilder::default()
        .per_second(10)
        .burst_size(50)
        .key_extractor(SmartIpKeyExtractor)
        .finish()
        .unwrap(),
    );

    let app = Router::new()
    .route("/robots.txt", get(|| async {
        Response::builder()
            .header("Content-Type", "text/plain")
            .body(Body::from(include_str!("../static/robots.txt")))
            .unwrap()
    }))
    .merge(routes::pages::router())
    .nest("/api", routes::api::router().layer(GovernorLayer::new(governor_config.clone())))
    .nest("/htmx", routes::htmx::router().layer(GovernorLayer::new(governor_config)))
    .nest_service("/static", ServeDir::new("crates/portfolio/static"))
    .nest_service("/wasm", ServeDir::new("wasm"))
    .layer(middleware::from_fn(log_request))
    .layer(TraceLayer::new_for_http())
    .layer(CompressionLayer::new())
    .layer(
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
    )
    .layer(SetResponseHeaderLayer::if_not_present(
        X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    ))
    .layer(SetResponseHeaderLayer::if_not_present(
        X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    ))
    .with_state(state);

    let addr = SocketAddr::new(config.host.parse()?, config.port);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Listening on https://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
