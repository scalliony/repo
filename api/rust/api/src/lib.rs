pub enum EntityType {
    Unexpected = -1,
    Rock,
    Bot,
    Building,
}
impl EntityType {
    fn from_i32(value: i32) -> EntityType {
        match value {
            0 => EntityType::Rock,
            1 => EntityType::Bot,
            2 => EntityType::Building,
            _ => EntityType::Unexpected,
        }
    }
}

pub enum Rotation {
    Up,
    Right,
    Down,
    Left,
}
impl Rotation {
    fn from_i32(value: i32) -> Rotation {
        match value {
            1 => Rotation::Right,
            2 => Rotation::Down,
            3 => Rotation::Left,
            _ => Rotation::Up,
        }
    }
}

pub struct Entity {
    pub id: i64,
    pub typ: EntityType,
}

pub mod io {
    #[link(wasm_import_module = "io")]
    extern "C" {
        /// imports io.log
        #[link_name = "log"]
        fn raw_io_log(s: *const u8, l: u32);
    }

    /// Write string to logs
    pub fn log(s: &str) {
        unsafe {
            raw_io_log(s.as_ptr(), s.len() as u32);
        }
    }
}

/// Write string formatted to logs
/// Warning: formatting is not cheap
#[macro_export]
macro_rules! log {
    ($($t:tt)*) => (::scalliony_api::console::log(&format_args!($($t)*).to_string()))
}

pub mod motor {
    #[link(wasm_import_module = "motor")]
    extern "C" {
        /// imports motor.rotate
        #[link_name = "rotate"]
        fn raw_motor_rotate(left: i32);

        /// imports motor.move
        #[link_name = "move"]
        fn raw_motor_move(dist: i32);
    }

    pub fn rotate(left: bool) {
        unsafe {
            raw_motor_rotate(match left {
                true => -1,
                false => 1,
            });
        }
    }
    pub fn rotate_left() {
        rotate(true);
    }
    pub fn rotate_right() {
        rotate(false);
    }

    /// Move forward of dist cells
    /// Direction depends of current rotation
    /// Actual movement is delayed
    pub fn go_forward(dist: u16) {
        unsafe {
            raw_motor_move(dist as i32);
        }
    }
}

pub mod sensors {
    #[repr(C)]
    struct Entity {
        id: i64,
        type_id: i32,
    }
    impl Entity {
        fn as_pub(&self) -> Option<crate::Entity> {
            match crate::EntityType::from_i32(self.type_id) {
                crate::EntityType::Unexpected => None,
                typ if self.id > 0 => Some(crate::Entity { id: self.id, typ }),
                _ => None,
            }
        }
    }

    #[link(wasm_import_module = "sensors")]
    extern "C" {
        /// imports sensors.contact_s
        #[link_name = "contact_s"]
        fn raw_sensors_contact() -> Entity;
    }

    /// Check for entity just in front (depending of rotation)
    /// Returns entity if something is in contact
    pub fn contact() -> Option<crate::Entity> {
        unsafe { raw_sensors_contact().as_pub() }
    }
}
