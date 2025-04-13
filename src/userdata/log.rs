/// A Lua userdata type that provides logging functionality.
///
/// This struct enables Lua scripts to log messages at various severity levels, including:
/// `ERROR`, `WARN`, `INFO`, `DEBUG`, and `TRACE`. It uses the `tracing` crate under the hood
/// for structured, performant logging.
///
/// # Example
/// Here's how to use the `Log` module in Lua:
///
/// ```lua
/// local log = require "log"
///
/// log.error("This is an error message.")
/// log.warn("This is a warning message.")
/// log.info("This is an informational message.")
/// log.debug("This is a debug message.")
/// log.trace("This is a trace message.")
///
/// -- Logging with a custom level:
/// log.log(log.INFO, "Custom info log.")
/// ```
///
/// The `log` function allows specifying a custom level, and shortcut methods
/// like `log.error` are available for convenience.
pub struct Log;

impl Log {
    /// Logs a message at the specified level.
    ///
    /// # Parameters
    /// - `level` (`&str`): The log level as a string. Must be one of: ERROR, WARN, INFO, DEBUG, TRACE.
    /// - `msg` (`S`): The message to log. Any type implementing `Display` is supported.
    ///
    /// # Returns
    /// - `Ok(())` if the message was logged successfully.
    /// - `Err(mlua::Error)` if the log level is invalid.
    pub fn log<S>(level: &str, msg: S) -> Result<(), mlua::Error>
    where
        S: std::fmt::Display,
    {
        let level: tracing::Level = level
            .parse()
            .map_err(|_| mlua::Error::external(anyhow::anyhow!("log level is invalid")))?;

        match level {
            tracing::Level::ERROR => tracing::error!("{msg}"),
            tracing::Level::WARN => tracing::warn!("{msg}"),
            tracing::Level::INFO => tracing::info!("{msg}"),
            tracing::Level::DEBUG => tracing::debug!("{msg}"),
            tracing::Level::TRACE => tracing::trace!("{msg}"),
        }

        Ok(())
    }

    /// Logs a message at the ERROR level.
    pub fn error<S>(msg: S) -> Result<(), mlua::Error>
    where
        S: std::fmt::Display,
    {
        Self::log(tracing::Level::ERROR.as_str(), msg)
    }

    /// Logs a message at the WARN level.
    pub fn warn<S>(msg: S) -> Result<(), mlua::Error>
    where
        S: std::fmt::Display,
    {
        Self::log(tracing::Level::WARN.as_str(), msg)
    }

    /// Logs a message at the INFO level.
    pub fn info<S>(msg: S) -> Result<(), mlua::Error>
    where
        S: std::fmt::Display,
    {
        Self::log(tracing::Level::INFO.as_str(), msg)
    }

    /// Logs a message at the DEBUG level.
    pub fn debug<S>(msg: S) -> Result<(), mlua::Error>
    where
        S: std::fmt::Display,
    {
        Self::log(tracing::Level::DEBUG.as_str(), msg)
    }

    /// Logs a message at the TRACE level.
    pub fn trace<S>(msg: S) -> Result<(), mlua::Error>
    where
        S: std::fmt::Display,
    {
        Self::log(tracing::Level::TRACE.as_str(), msg)
    }

    /// Formats a log message using Lua's string.format function.
    pub fn format(
        lua: &mlua::Lua,
        fmt: &str,
        vals: mlua::MultiValue,
    ) -> Result<String, mlua::Error> {
        lua.globals()
            .get::<mlua::Table>("string")
            .expect("get string table")
            .get::<mlua::Function>("format")
            .expect("get format function")
            .call::<String>((fmt, vals))
    }
}

impl mlua::UserData for Log {
    /// Adds fields for log levels to the `Log` struct for Lua use.
    ///
    /// These fields can be used as constants for specifying log levels in Lua:
    /// - `log.ERROR`
    /// - `log.WARN`
    /// - `log.INFO`
    /// - `log.DEBUG`
    /// - `log.TRACE`
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field("ERROR", "ERROR");
        fields.add_field("WARN", "WARN");
        fields.add_field("INFO", "INFO");
        fields.add_field("DEBUG", "DEBUG");
        fields.add_field("TRACE", "TRACE");
    }

    /// Adds logging methods to the `Log` struct for Lua use.
    ///
    /// Methods include:
    /// - `log(level, msg)`: Logs a message at the specified level.
    /// - `log.error(msg)`: Logs a message at the ERROR level.
    /// - `log.warn(msg)`: Logs a message at the WARN level.
    /// - `log.info(msg)`: Logs a message at the INFO level.
    /// - `log.debug(msg)`: Logs a message at the DEBUG level.
    /// - `log.trace(msg)`: Logs a message at the TRACE level.
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("log", |_lua, (level, msg): (String, String)| {
            Self::log(&level, &msg)
        });
        methods.add_function(
            "logf",
            |lua, (level, fmt, vals): (String, String, mlua::MultiValue)| {
                let msg = Self::format(lua, &fmt, vals)?;
                Self::log(&level, &msg)
            },
        );

        methods.add_function("error", |_lua, msg: String| Self::log("ERROR", &msg));
        methods.add_function("errorf", |lua, (fmt, vals): (String, mlua::MultiValue)| {
            let msg = Self::format(lua, &fmt, vals)?;
            Self::log("ERROR", &msg)
        });

        methods.add_function("warn", |_lua, msg: String| Self::log("WARN", &msg));
        methods.add_function("warnf", |lua, (fmt, vals): (String, mlua::MultiValue)| {
            let msg = Self::format(lua, &fmt, vals)?;
            Self::log("WARN", &msg)
        });

        methods.add_function("info", |_lua, msg: String| Self::log("INFO", &msg));
        methods.add_function("infof", |lua, (fmt, vals): (String, mlua::MultiValue)| {
            let msg = Self::format(lua, &fmt, vals)?;
            Self::log("INFO", &msg)
        });

        methods.add_function("debug", |_lua, msg: String| Self::log("DEBUG", &msg));
        methods.add_function("debugf", |lua, (fmt, vals): (String, mlua::MultiValue)| {
            let msg = Self::format(lua, &fmt, vals)?;
            Self::log("DEBUG", &msg)
        });

        methods.add_function("trace", |_lua, msg: String| Self::log("TRACE", &msg));
        methods.add_function("tracef", |lua, (fmt, vals): (String, mlua::MultiValue)| {
            let msg = Self::format(lua, &fmt, vals)?;
            Self::log("TRACE", &msg)
        });
    }
}
