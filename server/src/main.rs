mod api;
mod auth;
mod game;
mod http;
mod util;

#[tokio::main]
async fn main() {
    let _guard = util::install();

    let interface = game::run();

    let router =
        http::Router::new().nest("/api", api::router(interface.clone())).fallback(http::assets());
    let router = auth::nest(router, "/auth");

    http::serve(router, interface).await;
}
