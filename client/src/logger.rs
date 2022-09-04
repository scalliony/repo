use macroquad::logging::*;

#[inline(always)]
pub fn init() {
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::builder().filter_level(LevelFilter::Info).parse_default_env().init();

    #[cfg(target_arch = "wasm32")]
    {
        use macroquad::miniquad::log;

        pub struct MqLog;
        impl Log for MqLog {
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
        set_logger(&MqLog).unwrap();
        set_max_level(LevelFilter::max());
    }
}
