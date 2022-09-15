use crate::game;
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    response::IntoResponse,
    routing::{get_service, MethodRouter},
};
use std::net::SocketAddr;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    request_id::MakeRequestUuid, services::ServeDir, set_header::SetResponseHeader,
    trace::TraceLayer, ServiceBuilderExt,
};

pub type Router = axum::Router;

pub fn assets() -> MethodRouter {
    let dir = std::env::var("ASSETS_DIR").unwrap_or_else(|_| "dist".into());
    if !std::path::Path::new(&dir).join("index.html").is_file() {
        tracing::warn!("no 'ASSETS_DIR' at {}", dir)
    }
    let srv = SetResponseHeader::if_not_present(
        ServeDir::new(dir),
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("public, max-age=86400"),
    );
    get_service(srv).handle_error(handle_error)
}
async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}

pub async fn serve(app: axum::Router<Body>, interface: game::InterfaceRef) {
    let middleware = ServiceBuilder::new()
        //.timeout
        //.rate / concurrency limit
        //.request_body_limit
        .set_x_request_id(MakeRequestUuid)
        .layer(TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
            let id = request
                .headers()
                .get("x-request-id")
                .map(|h| h.to_str().unwrap_or_default())
                .unwrap_or_default();
            tracing::debug_span!(
                "request",
                method = %request.method(),
                uri = %request.uri(),
                id = %id,
            )
        }))
        //.compression
        .propagate_x_request_id();

    let commands = interface.commands.clone();
    let shutdown = async move {
        shutdown_signal().await;
        _ = commands.send(game::Command::ChangeState(game::State::Stopped));
    };

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::warn!("listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.layer(middleware).into_make_service())
        .with_graceful_shutdown(shutdown)
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::warn!("starting graceful shutdown");
}
