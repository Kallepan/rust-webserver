use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

pub struct Logger {
    stdout: Arc<Mutex<dyn Write + Send>>,
    stderr: Arc<Mutex<dyn Write + Send>>,
}

pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl Logger {
    pub fn new() -> Self {
        Logger {
            stdout: Arc::new(Mutex::new(io::stdout())),
            stderr: Arc::new(Mutex::new(io::stderr())),
        }
    }

    pub fn log(&self, level: LogLevel, message: &str) {
        let (level_str, mut output) = match level {
            LogLevel::Debug => ("DEBUG", self.stdout.lock().unwrap()),
            LogLevel::Info => ("INFO", self.stdout.lock().unwrap()),
            LogLevel::Warning => ("WARNING", self.stdout.lock().unwrap()),
            LogLevel::Error => ("ERROR", self.stderr.lock().unwrap()),
        };

        let timestamp = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => duration.as_secs(),
            Err(_) => 0,
        };

        match writeln!(output, "[{}] [{}] {}", timestamp, level_str, message) {
            Ok(_) => (),
            Err(err) => eprintln!("Failed to write to output: {}", err),
        }
    }
    pub fn info(&self, message: &str) {
        self.log(LogLevel::Info, message);
    }

    pub fn warn(&self, message: &str) {
        self.log(LogLevel::Warning, message);
    }

    pub fn error(&self, message: &str) {
        self.log(LogLevel::Error, message);
    }

    pub fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger() {
        let logger = Logger::new();
        logger.debug("debug message");
        logger.info("info message");
        logger.warn("warn message");
        logger.error("error message");
    }
}
