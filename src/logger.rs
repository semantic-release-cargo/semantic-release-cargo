use std::io::{self, Write};
use std::sync::Mutex;

use log::{Level, LevelFilter, Log};

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

const fn level_into_level_filter(level: Level) -> LevelFilter {
    match level {
        Level::Error => LevelFilter::Error,
        Level::Warn => LevelFilter::Warn,
        Level::Info => LevelFilter::Info,
        Level::Debug => LevelFilter::Debug,
        Level::Trace => LevelFilter::Trace,
    }
}

#[derive(Debug)]
#[allow(unused)]
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

/// A [Mutex]-wrapped dynamic [LogDestinationWriter]
type LockableLogDestinationWriter = Mutex<LogDestinationWriter>;

#[allow(unused)]
enum LogDestination {
    Single(LockableLogDestinationWriter),
    Multi(Vec<LockableLogDestinationWriter>),
}

impl LogDestination {
    #[allow(unused)]
    fn push<W>(self, writer: W) -> Self
    where
        W: Write + Send + Sync + Sized + 'static,
    {
        let dest = {
            let boxed_dest = Box::new(writer) as LogDestinationWriter;

            Mutex::new(boxed_dest)
        };

        match self {
            LogDestination::Single(inner) => Self::Multi(vec![inner, dest]),
            LogDestination::Multi(mut inner) => {
                inner.push(dest);
                Self::Multi(inner)
            }
        }
    }
}

impl<W> From<W> for LogDestination
where
    W: Write + Send + Sync + Sized + 'static,
{
    fn from(writer: W) -> Self {
        let dest = {
            let boxed_dest = Box::new(writer) as LogDestinationWriter;

            Mutex::new(boxed_dest)
        };

        LogDestination::Single(dest)
    }
}

/// A builder for the internal logger. This exposes a builder pattern for
/// setting all configurable options on the logger. Once finalized, init is
/// called to finalize the configuration and set it globally..
pub struct LoggerBuilder {
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
        let max_level_filter = self.logger.max_level;
        let boxed_logger = Box::new(self.logger);

        log::set_boxed_logger(boxed_logger)
            .map(|()| log::set_max_level(max_level_filter))
            .map_err(|_| Error::Initialization)
    }

    /// Set the error log level destination.
    fn set_error_dest(mut self, dest: LogDestination) -> Self {
        self.logger.error = dest;
        self
    }

    /// Set the warn log level destination.
    fn set_warn_dest(mut self, dest: LogDestination) -> Self {
        self.logger.warn = dest;
        self
    }

    /// Set the info log level destination.
    fn set_info_dest(mut self, dest: LogDestination) -> Self {
        self.logger.info = dest;
        self
    }

    /// Set the debug log level destination.
    fn set_debug_dest(mut self, dest: LogDestination) -> Self {
        self.logger.debug = dest;
        self
    }

    /// Set the trace log level destination.
    fn set_trace_dest(mut self, dest: LogDestination) -> Self {
        self.logger.trace = dest;
        self
    }

    /// Append a error log level destination writer.
    fn append_error_dest<W>(mut self, dest: W) -> Self
    where
        W: Write + Send + Sync + Sized + 'static,
    {
        self.logger.error = self.logger.error.push(dest);
        self
    }

    /// Append a warn log level destination writer.
    fn append_warn_dest<W>(mut self, dest: W) -> Self
    where
        W: Write + Send + Sync + Sized + 'static,
    {
        self.logger.warn = self.logger.warn.push(dest);
        self
    }

    /// Append a info log level destination writer.
    fn append_info_dest<W>(mut self, dest: W) -> Self
    where
        W: Write + Send + Sync + Sized + 'static,
    {
        self.logger.info = self.logger.info.push(dest);
        self
    }

    /// Append a debug log level destination writer.
    fn append_debug_dest<W>(mut self, dest: W) -> Self
    where
        W: Write + Send + Sync + Sized + 'static,
    {
        self.logger.debug = self.logger.debug.push(dest);
        self
    }

    /// Append a trace log level destination writer.
    fn append_trace_dest<W>(mut self, dest: W) -> Self
    where
        W: Write + Send + Sync + Sized + 'static,
    {
        self.logger.trace = self.logger.trace.push(dest);
        self
    }

    /// Set the output destination for a given log level.
    pub fn output<W>(self, level: Level, dest: W) -> Self
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

    /// Append an output destination for a given log level.
    #[allow(unused)]
    pub fn append_output<W>(self, level: Level, dest: W) -> Self
    where
        W: Write + Send + Sync + Sized + 'static,
    {
        match level {
            Level::Error => self.append_error_dest(dest),
            Level::Warn => self.append_warn_dest(dest),
            Level::Info => self.append_info_dest(dest),
            Level::Debug => self.append_debug_dest(dest),
            Level::Trace => self.append_trace_dest(dest),
        }
    }

    /// Sets the maximum log level explicitly to the value passed.
    pub fn max_level(mut self, max_level: Level) -> Self {
        self.logger.max_level = level_into_level_filter(max_level);

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

        let adjusted_max_level_filter = level_into_level_filter(adjusted_max_level);
        self.logger.max_level = adjusted_max_level_filter;

        self
    }
}

impl Default for LoggerBuilder {
    fn default() -> Self {
        let logger = Logger {
            max_level: level_into_level_filter(Self::DEFAULT_LOG_LEVEL),
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
    max_level: LevelFilter,
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

    #[allow(unused)]
    pub fn max_level_filter(&self) -> LevelFilter {
        self.max_level
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.max_level
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let record_level = record.level();
        let level_oriented_log_destination = self.as_logdestination_from_level(record_level);

        match level_oriented_log_destination {
            LogDestination::Single(writer) => {
                if let Ok(mut log_writer) = writer.lock() {
                    let _ = writeln!(log_writer, "{}", record.args());
                }
            }
            LogDestination::Multi(writers) => {
                let lockable_writers = writers
                    .iter()
                    .flat_map(|lockable_writer| lockable_writer.lock());

                for mut log_writer in lockable_writers {
                    let _ = writeln!(log_writer, "{}", record.args());
                }
            }
        }
    }

    fn flush(&self) {}
}

#[cfg(test)]
mod tests {}
