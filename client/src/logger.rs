#[inline]
pub fn init() {
    #[cfg(feature = "log-rs")]
    {
        #[cfg(target_arch = "wasm32")]
        mini::RawLog::init();
        #[cfg(not(target_arch = "wasm32"))]
        env_logger::builder()
            .filter_level(macroquad::logging::LevelFilter::Info)
            .parse_default_env()
            .init();
    }
}

#[cfg(all(feature = "log-rs", target_arch = "wasm32"))]
mod mini {
    use macroquad::logging::*;
    use macroquad::miniquad::log;

    pub struct RawLog;
    impl RawLog {
        pub fn init() {
            set_boxed_logger(Box::new(Self)).unwrap()
        }
    }
    impl Log for RawLog {
        fn enabled(&self, _: &Metadata) -> bool {
            true
        }
        fn log(&self, record: &Record) {
            let lvl = match record.level() {
                Level::Error => log::Level::Error,
                Level::Warn => log::Level::Warn,
                Level::Info => log::Level::Info,
                Level::Debug => log::Level::Debug,
                Level::Trace => log::Level::Trace,
            };
            log!(lvl, "[{}] {}", record.target(), record.args());
        }
        fn flush(&self) {}
    }
}
