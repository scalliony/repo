use tracing_appender::non_blocking;
use tracing_subscriber::{filter::LevelFilter, prelude::*, EnvFilter};

pub fn install() -> non_blocking::WorkerGuard {
    _ = dotenv::dotenv();

    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    let (_non_blocking_writer, guard) = non_blocking(std::io::stdout());
    #[cfg(not(feature = "log-tree"))]
    let layer = tracing_subscriber::fmt::layer().with_writer(_non_blocking_writer);
    #[cfg(feature = "log-tree")]
    let layer = tracing_forest::ForestLayer::default();

    tracing_subscriber::registry()
        .with(filter)
        .with(layer)
        .init();
    guard
}
