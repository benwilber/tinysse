use clap::Parser;
use humantime::parse_duration;
use std::{net::SocketAddr, path::PathBuf, time::Duration};
use tracing::Level;

/// Tiny SSE Server
///
/// This server supports Lua scripting for customization. Use the following options
/// to configure the server.
///
/// Duration Format:
/// Time-related options (e.g., keep-alive, timeout) use a human-readable format:
/// - `1s` means 1 second.
/// - `1000ms` means 1000 milliseconds (can be shortened to `1s`).
/// - Other examples: `5m` (5 minutes), `2h` (2 hours), `3d` (3 days).
///
/// Use these formats consistently for options like `--keep-alive`, `--timeout`, etc.
#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(
        short,
        long,
        value_name = "ADDR:PORT",
        default_value = "127.0.0.1:1983",
        env = "TINYSSE_LISTEN",
        help = "The address and port for the HTTP server to listen on"
    )]
    pub listen: SocketAddr,

    #[clap(
        short = 'L',
        long,
        value_name = "LEVEL",
        default_value = "INFO",
        env = "TINYSSE_LOG_LEVEL",
        help = "The logging level for the server. Possible values: ERROR, WARN, INFO, DEBUG, TRACE"
    )]
    pub log_level: Level,

    #[clap(
        short,
        long,
        value_name = "INTERVAL",
        default_value = "60s",
        value_parser = parse_duration,
        env = "TINYSSE_KEEP_ALIVE",
        help = "The interval between keep-alive messages sent to clients (e.g., 60s, 2m).\n\
                Keep-alive messages are sent periodically to ensure that clients remain connected."
    )]
    pub keep_alive: Duration,

    #[clap(
        short = 'K',
        long,
        value_name = "TEXT",
        default_value = "keep-alive",
        env = "TINYSSE_KEEP_ALIVE_TEXT",
        help = "The text content of the keep-alive messages sent to clients.\n\
                This text helps clients recognize keep-alive messages and avoid treating them as real events."
    )]
    pub keep_alive_text: String,

    #[clap(
        short,
        long,
        value_name = "TIMEOUT",
        default_value = "5m",
        value_parser = parse_duration,
        env = "TINYSSE_TIMEOUT",
        help = "The timeout duration for idle connections (e.g., 5m, 300s, 10m).\n\
                Connections that remain idle longer than this duration will be closed."
    )]
    pub timeout: Duration,

    #[clap(
        short = 'r',
        long,
        value_name = "RETRY",
        default_value = "0s",
        value_parser = parse_duration,
        env = "TINYSSE_TIMEOUT_RETRY",
        help = "The retry interval sent to clients after a connection timeout (e.g., 0s, 2s).\n\
                This interval instructs clients how long to wait before attempting to reconnect."
    )]
    pub timeout_retry: Duration,

    #[clap(
        short = 'c',
        long,
        value_name = "CAPACITY",
        default_value = "32",
        env = "TINYSSE_CAPACITY",
        help = "The capacity of the server's internal event queue"
    )]
    pub capacity: usize,

    #[clap(
        short = 's',
        long,
        value_name = "PATH",
        env = "TINYSSE_SCRIPT",
        help = "The path to a Lua script for server customization"
    )]
    pub script: Option<PathBuf>,

    #[clap(
        long,
        env = "TINYSSE_UNSAFE_SCRIPT",
        default_value = "false",
        help = "Allow the Lua script to load (require) native code, such as shared (.so) libraries. \
                Enabling this can pose security risks, as native code can execute arbitrary operations. \
                Use this option only if you trust the Lua script and need it to load native modules."
    )]
    pub unsafe_script: bool,
}
