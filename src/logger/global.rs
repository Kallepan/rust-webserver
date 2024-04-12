use lazy_static::lazy_static;
use std::sync::Mutex;

use super::log::Logger;

// setup a global logger
lazy_static! {
    pub static ref LOGGER: Mutex<Logger> = Mutex::new(Logger::new());
}

// define macros for logging
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::logger::global::LOGGER.lock().unwrap().info(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::logger::global::LOGGER.lock().unwrap().warn(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::logger::global::LOGGER.lock().unwrap().error(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::logger::global::LOGGER.lock().unwrap().debug(&format!($($arg)*));
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_logger() {
        debug!("debug message");
        info!("info message");
        warn!("warn message");
        error!("error message");
    }

    #[test]
    fn test_logger_threaded() {
        use std::thread;

        let handles: Vec<_> = (0..10)
            .map(|i| {
                thread::spawn(move || {
                    debug!("debug message {}", i);
                    info!("info message {}", i);
                    warn!("warn message {}", i);
                    error!("error message {}", i);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
