use crate::game;
use axum::{body::Body, http::Request};
use axum_extra::routing::SpaRouter;
use std::net::SocketAddr;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{request_id::MakeRequestUuid, trace::TraceLayer, ServiceBuilderExt};

pub type Router = axum::Router;

pub fn spa() -> SpaRouter {
    let dir = std::env::var("ASSETS_DIR").unwrap_or_else(|_| "dist".into());
    if !std::path::Path::new(&dir).join("index.html").is_file() {
        tracing::warn!("no 'ASSETS_DIR' at {}", dir)
    }
    SpaRouter::new("/assets", dir)
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
