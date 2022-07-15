pub mod io {
    #[link(wasm_import_module = "io")]
    extern "C" {
        /// imports io.log
        #[link_name = "log"]
        fn _log(s: *const u8, l: u32);
    }

    /// Write string to logs
    pub fn log(s: &str) {
        unsafe {
            _log(s.as_ptr(), s.len() as u32);
        }
    }
}

/// Write string formatted to logs
/// Warning: formatting is not cheap
#[macro_export]
macro_rules! log {
    ($($t:tt)*) => (self::io::log(&format_args!($($t)*).to_string()))
}

pub mod motor {
    #[link(wasm_import_module = "motor")]
    extern "C" {
        /// imports motor.forward
        #[link_name = "forward"]
        fn _forward();

        /// imports motor.left
        #[link_name = "left"]
        fn _left();

        /// imports motor.right
        #[link_name = "right"]
        fn _right();
    }

    pub fn forward() {
        unsafe { _forward() };
    }
    pub fn left() {
        unsafe { _left() };
    }
    pub fn right() {
        unsafe { _right() };
    }
}

pub mod sensors {
    #[link(wasm_import_module = "sensors")]
    extern "C" {
        /// imports sensors.contact
        #[link_name = "contact"]
        fn _contact() -> i32;
    }

    /// Check if there is something blocking path just in front (depending of rotation)
    pub fn contact() -> bool {
        unsafe { _contact() > 0 }
    }
}
