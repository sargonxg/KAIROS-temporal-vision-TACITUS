use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderValue, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use kairos_core::{validate_graph, AnalysisRequest, Kairos, ValidationRequest};
use rust_embed::RustEmbed;
use std::env;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[derive(RustEmbed)]
#[folder = "../../web"]
struct WebAssets;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,kairos=debug".into()),
        )
        .json()
        .init();

    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/api/analyze", post(analyze))
        .route("/api/v1/analyze", post(analyze))
        .route("/api/v1/graph", post(graph))
        .route("/api/v1/validate", post(validate))
        .fallback(static_handler)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(Kairos::from_env());

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{port}").parse()?;
    tracing::info!(%addr, "kairos-server listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn validate(Json(req): Json<ValidationRequest>) -> impl IntoResponse {
    (StatusCode::OK, Json(validate_graph(&req))).into_response()
}

async fn analyze(
    State(kairos): State<Kairos>,
    Json(req): Json<AnalysisRequest>,
) -> impl IntoResponse {
    match kairos.analyze(req).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => {
            tracing::warn!(error = %err, "analysis failed");
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": err.to_string()})),
            )
                .into_response()
        }
    }
}

async fn graph(
    State(kairos): State<Kairos>,
    Json(req): Json<AnalysisRequest>,
) -> impl IntoResponse {
    match kairos.analyze(req).await {
        Ok(result) => (StatusCode::OK, Json(result.graph)).into_response(),
        Err(err) => {
            tracing::warn!(error = %err, "graph export failed");
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": err.to_string()})),
            )
                .into_response()
        }
    }
}

async fn static_handler(uri: Uri) -> Response {
    let path = match uri.path() {
        "/" => "index.html",
        p => p.trim_start_matches('/'),
    };
    match WebAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            let mut res = Response::new(Body::from(content.data.into_owned()));
            res.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str(mime.as_ref())
                    .unwrap_or(HeaderValue::from_static("application/octet-stream")),
            );
            res
        }
        None => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}
