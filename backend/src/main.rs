use axum::{
    handler::HandlerWithoutStateExt,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::path::PathBuf;
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::{
    compression::CompressionLayer, services::ServeDir, timeout::TimeoutLayer, trace::TraceLayer,
};

struct Env {
    site_addr: String,
    dist_dir: PathBuf,
}

impl Env {
    fn get_or_default() -> Self {
        let site_addr = std::env::var("SITE_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
        let dist_dir = std::env::var("DIST_DIR")
            .unwrap_or_else(|_| format!("{}/../frontend/dist", env!("CARGO_MANIFEST_DIR")))
            .into();

        Self {
            site_addr,
            dist_dir,
        }
    }
}

async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, Html("<h1>404 Not Found</h1>"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let Env {
        site_addr,
        dist_dir,
    } = Env::get_or_default();

    let router = Router::new()
        // test api
        .route("/api/test", get(|| async { "hello from axum" }))
        // serve the frontend statically
        .fallback_service(ServeDir::new(&dist_dir).not_found_service(not_found.into_service()))
        .layer(CompressionLayer::new().gzip(true))
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(TraceLayer::new_for_http());

    let listener = TcpListener::bind(&site_addr).await?;

    tracing::info!("Listening on http://{site_addr}/");
    tracing::info!("Serving files in {}", dist_dir.display());
    axum::serve(listener, router).await?;
    Ok(())
}
