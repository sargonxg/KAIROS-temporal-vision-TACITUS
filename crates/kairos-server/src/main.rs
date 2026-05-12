use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderValue, Request, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::prelude::{Engine, BASE64_STANDARD};
use kairos_core::{validate_graph, AnalysisRequest, Kairos, ValidationRequest};
use rust_embed::RustEmbed;
use std::convert::Infallible;
use std::env;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tower_service::Service;
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
        .layer(AuthLayer::from_env())
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

#[derive(Clone, Debug)]
struct AuthLayer {
    credentials: Option<AuthCredentials>,
}

#[derive(Clone, Debug)]
struct AuthCredentials {
    username: String,
    password: String,
}

impl AuthLayer {
    fn from_env() -> Self {
        let username = env::var("KAIROS_BASIC_USER")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let password = env::var("KAIROS_BASIC_PASSWORD")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let credentials = match (username, password) {
            (Some(username), Some(password)) => Some(AuthCredentials { username, password }),
            _ => None,
        };
        Self { credentials }
    }
}

impl<S> tower::Layer<S> for AuthLayer {
    type Service = AuthService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthService {
            inner,
            credentials: self.credentials.clone(),
        }
    }
}

#[derive(Clone)]
struct AuthService<S> {
    inner: S,
    credentials: Option<AuthCredentials>,
}

impl<S> Service<Request<Body>> for AuthService<S>
where
    S: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let credentials = self.credentials.clone();
        let mut inner = self.inner.clone();
        Box::pin(async move {
            if req.uri().path() == "/healthz" || credentials.is_none() {
                return inner.call(req).await;
            }
            let credentials = credentials.expect("checked above");
            if authorized(req.headers().get(header::AUTHORIZATION), &credentials) {
                return inner.call(req).await;
            }
            Ok(unauthorized_response())
        })
    }
}

fn authorized(header_value: Option<&HeaderValue>, credentials: &AuthCredentials) -> bool {
    let Some(header_value) = header_value.and_then(|value| value.to_str().ok()) else {
        return false;
    };
    let Some(encoded) = header_value.strip_prefix("Basic ") else {
        return false;
    };
    let Ok(decoded) = BASE64_STANDARD.decode(encoded.trim()) else {
        return false;
    };
    let Ok(pair) = String::from_utf8(decoded) else {
        return false;
    };
    let expected = format!("{}:{}", credentials.username, credentials.password);
    constant_time_eq(pair.as_bytes(), expected.as_bytes())
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let diff = left
        .iter()
        .zip(right.iter())
        .fold(0_u8, |acc, (a, b)| acc | (a ^ b));
    diff == 0
}

fn unauthorized_response() -> Response {
    let mut res = (StatusCode::UNAUTHORIZED, "authentication required").into_response();
    res.headers_mut().insert(
        header::WWW_AUTHENTICATE,
        HeaderValue::from_static("Basic realm=\"KAIROS\", charset=\"UTF-8\""),
    );
    res
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
