//! Crude logging utility for chisel.

/// The global log level. And probably one of the few acceptable uses of mutable globals.
static mut LOG_LEVEL: i32 = 0;

#[macro_export]
macro_rules! chisel_debug {
    ($lvl:expr, $($arg:tt)*) => {
        crate::logger::Logger::with_global_level().log($lvl, &format!($($arg)*));
    }
}

/// Simple logging utility struct.
pub struct Logger(i32);

impl Logger {
    pub fn with_global_level() -> Self {
        unsafe { Logger(LOG_LEVEL) }
    }

    pub fn log<T: AsRef<str>>(&self, level: i32, message: T) {
        if self.0 >= level {
            eprintln!("{}", message.as_ref());
        }
    }
}

/// Set the global log level.
// NOTE: Unsafe in a multithreaded context. Add mutex later when this is moved into the library.
pub fn set_global_log_level(lvl: i32) {
    unsafe {
        LOG_LEVEL = lvl;
    }
}
