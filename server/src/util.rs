use tracing_appender::non_blocking;
use tracing_subscriber::{filter::LevelFilter, prelude::*, EnvFilter};

pub fn install() -> non_blocking::WorkerGuard {
    _ = dotenv::dotenv();

    let (non_blocking_writer, guard) = non_blocking(std::io::stdout());
    tracing_subscriber::registry()
        .with(
            EnvFilter::builder().with_default_directive(LevelFilter::INFO.into()).from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking_writer))
        .init();
    guard
}
