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

#[derive(Debug)]
pub enum Error {
    Initialization,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initialization => writeln!(f, "unable to initialize logger"),
        }
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
    /// Acquire a mutex on the destination. Blocking on the current thread
    /// until it is able to do so.
    pub fn lock(&self) -> std::sync::LockResult<std::sync::MutexGuard<'_, LogDestinationWriter>> {
        self.dest.lock()
    }

    /// Attempt to acquire the lock.
    ///
    /// If it is unable to do so, an `Err` is returned.
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

/// A builder for the internal logger. This exposes a builder pattern for
/// setting all configurable options on the logger. Once finalized, init is
/// called to finalize the configuration and set it globally..
pub(crate) struct LoggerBuilder {
    logger: Logger,
}

impl LoggerBuilder {
    // A mirror of the module level default rescoped to a static logger constant.
    const DEFAULT_LOG_LEVEL: Level = DEFAULT_LOG_LEVEL;
}

impl LoggerBuilder {
    /// Finalizes a [Logger]'s configuration and assigns the global logger to
    /// the current settings.
    ///
    /// # Errors
    /// An error is returned if this is already set. Caller must guarantee this
    /// is called no more than once.
    #[allow(unused)]
    pub fn init(self) -> Result<(), Error> {
        let boxed_logger = Box::new(self.logger);

        log::set_boxed_logger(boxed_logger).map_err(|_| Error::Initialization)
    }

    /// Set the error log level destination
    fn set_error_dest(mut self, dest: LogDestination) -> Self {
        self.logger.error = dest;
        self
    }

    /// Set the warn log level destination
    fn set_warn_dest(mut self, dest: LogDestination) -> Self {
        self.logger.warn = dest;
        self
    }

    /// Set the info log level destination
    fn set_info_dest(mut self, dest: LogDestination) -> Self {
        self.logger.warn = dest;
        self
    }

    /// Set the debug log level destination
    fn set_debug_dest(mut self, dest: LogDestination) -> Self {
        self.logger.debug = dest;
        self
    }

    /// Set the trace log level destination
    fn set_trace_dest(mut self, dest: LogDestination) -> Self {
        self.logger.trace = dest;
        self
    }

    /// Set the output destination for a given log level.
    #[allow(unused)]
    pub fn output<W>(mut self, level: Level, dest: W) -> Self
    where
        W: Write + Send + Sync + Sized + 'static,
    {
        let log_destination = LogDestination::from(dest);
        match level {
            Level::Error => self.set_error_dest(log_destination),
            Level::Warn => self.set_warn_dest(log_destination),
            Level::Info => self.set_info_dest(log_destination),
            Level::Debug => self.set_debug_dest(log_destination),
            Level::Trace => self.set_trace_dest(log_destination),
        }
    }

    /// Sets the maximum log level explicitly to the value passed.
    #[allow(unused)]
    pub(crate) fn max_level(mut self, max_level: Level) -> Self {
        self.logger.max_level = max_level;

        self
    }

    /// Sets the log level based off the number of verbosity flags are passed.
    /// The verbosity argument functions as an offset from the default log
    /// level where a value of `0` represents the default. Any value exceeding
    /// the offset of `Trace`, will be counted as `Trace`.
    #[allow(unused)]
    pub(crate) fn verbosity(mut self, verbosity: u8) -> Self {
        let verbosity = verbosity as usize;

        // The new verbosity offset from the default log level.
        let offset = (DEFAULT_LOG_LEVEL as usize) + verbosity;

        let adjusted_max_level = match offset {
            // there should be no case where 0 will occur, but this is for the
            // sake of being explicit.
            0 => unreachable!(),
            1 => Level::Error,
            2 => Level::Warn,
            3 => Level::Info,
            4 => Level::Debug,
            _ => Level::Trace,
        };

        self.logger.max_level = adjusted_max_level;

        self
    }
}

impl Default for LoggerBuilder {
    fn default() -> Self {
        let logger = Logger {
            max_level: Self::DEFAULT_LOG_LEVEL,
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
    max_level: Level,
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
        metadata.level() >= self.max_level
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
