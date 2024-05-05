use std::io::{self, Write};
use std::sync::Mutex;

use log::{Level, Log};

/// The default log level to use if no other is specified
const DEFAULT_LOG_LEVEL: Level = Level::Warn;

/// Return a default writer for a given level. In this case stderr for warn
/// and error and stdout for all others.
fn default_log_dest_for_level(level: Level) -> LogDestination {
    match level {
        Level::Error | Level::Warn => LogDestination::from(io::stderr()),
        Level::Info | Level::Debug | Level::Trace => LogDestination::from(io::stdout()),
    }
}

/// LogDestinationWriter represents a Boxed dynamic trait object for Writing
/// logs to. This will exclusively be exposed behind a [Mutex] and assigned to.
/// an owning [Logger]
type LogDestinationWriter = Box<dyn Write + Send + Sync>;

struct LogDestination {
    dest: Mutex<LogDestinationWriter>,
}

// Delegate methods for the inner mutex
impl LogDestination {
    pub fn lock(&self) -> std::sync::LockResult<std::sync::MutexGuard<'_, LogDestinationWriter>> {
        self.dest.lock()
    }

    #[allow(unused)]
    pub fn try_lock(
        &self,
    ) -> std::sync::TryLockResult<std::sync::MutexGuard<'_, LogDestinationWriter>> {
        self.dest.try_lock()
    }
}

impl<W> From<W> for LogDestination
where
    W: Write + Send + Sync + Sized + 'static,
{
    fn from(writer: W) -> Self {
        let dest = {
            let boxed_dest = Box::new(writer) as Box<dyn Write + Send + Sync>;

            Mutex::new(boxed_dest)
        };

        LogDestination { dest }
    }
}

pub(crate) struct LoggerBuilder {
    logger: Logger,
}

impl LoggerBuilder {
    // A mirror of the module level default rescoped to a static logger constant.
    const DEFAULT_LOG_LEVEL: Level = DEFAULT_LOG_LEVEL;
}

impl LoggerBuilder {
    #[allow(unused)]
    pub fn init(self) -> Result<(), String> {
        let boxed_logger = Box::new(self.logger);

        log::set_boxed_logger(boxed_logger).map_err(|e| e.to_string())
    }
}

impl Default for LoggerBuilder {
    fn default() -> Self {
        let logger = Logger {
            level: Self::DEFAULT_LOG_LEVEL,
            error: default_log_dest_for_level(Level::Error),
            warn: default_log_dest_for_level(Level::Warn),
            info: default_log_dest_for_level(Level::Info),
            debug: default_log_dest_for_level(Level::Debug),
            trace: default_log_dest_for_level(Level::Trace),
        };

        LoggerBuilder { logger }
    }
}

/// A generic logger type that allows an arbitrary destination for each level.
#[allow(unused)]
struct Logger {
    level: Level,
    error: LogDestination,
    warn: LogDestination,
    info: LogDestination,
    debug: LogDestination,
    trace: LogDestination,
}

impl Logger {
    fn as_logdestination_from_level(&self, level: Level) -> &LogDestination {
        match level {
            Level::Error => &self.error,
            Level::Warn => &self.warn,
            Level::Info => &self.info,
            Level::Debug => &self.debug,
            Level::Trace => &self.trace,
        }
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() >= self.level
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let record_level = record.level();
        let level_oriented_log_destination = self.as_logdestination_from_level(record_level);

        if let Ok(mut log_writer) = level_oriented_log_destination.lock() {
            let _ = writeln!(
                log_writer,
                "{}:{} -- {}",
                record.level(),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

#[cfg(test)]
mod tests {}
