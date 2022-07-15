mod api;
mod auth;
mod game;
mod http;
mod util;

#[tokio::main]
async fn main() {
    let _guard = util::install();

    let interface = game::run();

    let router = http::Router::new()
        // routes are matched from bottom to top
        .merge(http::spa())
        .nest("/auth", auth::router())
        .nest("/api", api::router(interface.clone()));

    http::serve(router, interface).await;
}
