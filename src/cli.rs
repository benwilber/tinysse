use bytesize::ByteSize;
use clap::Parser;
use http::{HeaderName, HeaderValue, Method};
use humantime::parse_duration;
use mlua::LuaSerdeExt;
use std::{net::SocketAddr, path::PathBuf, time::Duration};
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin};

/// Tiny SSE
///
/// A programmable server for Server-Sent Events (SSE).
#[derive(Debug, Clone, Parser)]
#[command(version)]
pub struct Cli {
    #[clap(
        short,
        long,
        value_name = "ADDR:PORT",
        default_value = "127.0.0.1:1983",
        env = "TINYSSE_LISTEN",
        help = "The address and port for the HTTP server to listen"
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
    pub log_level: tracing::Level,

    #[clap(
        short,
        long,
        value_name = "INTERVAL",
        default_value = "60s",
        value_parser = parse_duration,
        env = "TINYSSE_KEEP_ALIVE",
        help = "The interval between keep-alive messages sent to clients (e.g., 60s, 2m).\n\
                Keep-alive messages are sent periodically to ensure that clients remain connected"
    )]
    pub keep_alive: Duration,

    #[clap(
        short = 'K',
        long,
        value_name = "TEXT",
        default_value = "keep-alive",
        env = "TINYSSE_KEEP_ALIVE_TEXT",
        help = "The text of the keep-alive comment sent to clients."
    )]
    pub keep_alive_text: String,

    #[clap(
        short,
        long,
        value_name = "TIMEOUT",
        default_value = "5m",
        value_parser = parse_duration,
        env = "TINYSSE_TIMEOUT",
        help = "The timeout duration for subscriber connections (e.g., 5m, 300s, 10m).\n\
                Connections open for longer than this duration will be closed"
    )]
    pub timeout: Duration,

    #[clap(
        short = 'r',
        long,
        value_name = "RETRY",
        default_value = "0s",
        value_parser = parse_duration,
        env = "TINYSSE_TIMEOUT_RETRY",
        help = "The retry delay sent to clients after a connection timeout (e.g., 0s, 2s).\n\
                This delay instructs clients how long to wait before attempting to reconnect.\n\
                Setting this to 0s instructs the client to reconnect immediately"
    )]
    pub timeout_retry: Duration,

    #[clap(
        short = 'c',
        long,
        value_name = "CAPACITY",
        default_value = "256",
        env = "TINYSSE_CAPACITY",
        help = "The capacity of the server's internal message queue"
    )]
    pub capacity: usize,

    #[clap(
        short = 's',
        long,
        value_name = "FILE_PATH",
        env = "TINYSSE_SCRIPT",
        help = "The path to a Lua script for server customization"
    )]
    pub script: Option<PathBuf>,

    #[clap(
        long,
        value_name = "DATA",
        env = "TINYSSE_SCRIPT_DATA",
        help = "Optional data to pass to the Lua script as the `cli.script_data` value in the `startup(cli)` function"
    )]
    pub script_data: Option<String>,

    #[clap(
        long,
        value_name = "INTERVAL",
        default_value = "500ms",
        value_parser = parse_duration,
        env = "TINYSSE_SCRIPT_TICK",
        help = "The interval between Lua script ticks (e.g., 1s, 500ms). The script tick is a periodic event that allows the Lua script to perform background tasks in the `tick(count)` function"
    )]
    pub script_tick: Duration,

    #[clap(
        long,
        env = "TINYSSE_UNSAFE_SCRIPT",
        help = "Allow the Lua script to load (require) native code, such as shared (.so) libraries. \
                Enabling this can pose security risks, as native code can execute arbitrary operations. \
                Use this option only if you trust the Lua script and need it to load native modules"
    )]
    pub unsafe_script: bool,

    #[clap(
        short = 'm',
        long,
        value_name = "BYTES",
        default_value = "64KB",
        env = "TINYSSE_MAX_BODY_SIZE",
        help = "The maximum size of the publish request body that the server will accept (e.g., 32KB, 1MB)"
    )]
    pub max_body_size: ByteSize,

    #[clap(
        short = 'P',
        long,
        value_name = "URL_PATH",
        default_value = "/sse",
        env = "TINYSSE_PUB_PATH",
        help = "The URL path for publishing messages via POST"
    )]
    pub pub_path: String,

    #[clap(
        short = 'S',
        long,
        value_name = "URL_PATH",
        default_value = "/sse",
        env = "TINYSSE_SUB_PATH",
        help = "The URL path for subscribing to messages via GET"
    )]
    pub sub_path: String,

    #[clap(
        short = 'D',
        long,
        value_name = "DIR_PATH",
        env = "TINYSSE_SERVE_STATIC_DIR",
        help = "Serve static files from the specified directory under the path specified by `--serve-static-path`"
    )]
    pub serve_static_dir: Option<PathBuf>,

    #[clap(
        short = 'U',
        long,
        value_name = "URL_PATH",
        default_value = "/",
        env = "TINYSSE_SERVE_STATIC_PATH",
        help = "The URL path under which to serve static files from the directory specified by `--serve-static-dir`"
    )]
    pub serve_static_path: String,

    #[clap(
        long,
        value_name = "ORIGINS",
        default_value = "*",
        value_parser = parse_allow_origin,
        env = "TINYSSE_CORS_ALLOW_ORIGIN",
        help = "Set Access-Control-Allow-Origin header to the specified origin(s)"
    )]
    pub cors_allow_origin: AllowOrigin,

    #[clap(
        long,
        value_name = "METHODS",
        default_value = "GET, HEAD, POST",
        value_parser = parse_allow_methods,
        env = "TINYSSE_CORS_ALLOW_METHODS",
        help = "Set Access-Control-Allow-Methods header to the specified method(s)"
    )]
    pub cors_allow_methods: AllowMethods,

    #[clap(
        long,
        value_name = "HEADERS",
        default_value = "*",
        value_parser = parse_allow_headers,
        env = "TINYSSE_CORS_ALLOW_HEADERS",
        help = "Set Access-Control-Allow-Headers header to the specified header(s). (e.g., Cookie,Authorization)"
    )]
    pub cors_allow_headers: AllowHeaders,

    #[clap(
        long,
        env = "TINYSSE_CORS_ALLOW_CREDENTIALS",
        help = r#"Set Access-Control-Allow-Credentials header to true. Cannot be set if Access-Control-Allow-Origin or Access-Control-Allow-Headers is set to '*' (any)"#
    )]
    pub cors_allow_credentials: bool,

    #[clap(
        long,
        value_name = "DURATION",
        default_value = "0s",
        value_parser = parse_duration,
        env = "TINYSSE_CORS_MAX_AGE",
        help = "Set Access-Control-Max-Age header to the specified duration (e.g., 1h, 60s)"
    )]
    pub cors_max_age: Duration,
}

impl mlua::IntoLua for Cli {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        let tbl = lua.create_table()?;

        tbl.set("listen", self.listen.to_string())?;
        tbl.set("log_level", self.log_level.to_string())?;
        tbl.set("keep_alive", self.keep_alive.as_millis())?;
        tbl.set("keep_alive_text", self.keep_alive_text)?;
        tbl.set("timeout", self.timeout.as_millis())?;
        tbl.set("timeout_retry", self.timeout_retry.as_millis())?;
        tbl.set("capacity", self.capacity)?;
        tbl.set(
            "script",
            self.script
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned()),
        )?;
        tbl.set("script_tick", self.script_tick.as_millis())?;
        tbl.set("script_data", self.script_data)?;
        tbl.set("unsafe_script", self.unsafe_script)?;
        tbl.set("pub_path", self.pub_path)?;
        tbl.set("sub_path", self.sub_path)?;
        tbl.set(
            "serve_static_dir",
            self.serve_static_dir
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned()),
        )?;
        tbl.set("serve_static_path", self.serve_static_path)?;

        lua.to_value(&tbl)
    }
}

fn parse_allow_origin(s: &str) -> anyhow::Result<AllowOrigin> {
    if s.trim() == "*" {
        Ok(AllowOrigin::any())
    } else {
        Ok(AllowOrigin::list(
            s.split(',')
                .filter_map(|s| HeaderValue::from_str(s.trim()).ok()),
        ))
    }
}

fn parse_allow_headers(s: &str) -> anyhow::Result<AllowHeaders> {
    if s.trim() == "*" {
        Ok(AllowHeaders::any())
    } else {
        Ok(AllowHeaders::list(s.split(',').filter_map(|s| {
            HeaderName::from_bytes(s.trim().as_bytes()).ok()
        })))
    }
}

fn parse_allow_methods(s: &str) -> anyhow::Result<AllowMethods> {
    if s.trim() == "*" {
        Ok(AllowMethods::any())
    } else {
        Ok(AllowMethods::list(s.split(',').filter_map(|s| {
            Method::from_bytes(s.trim().as_bytes()).ok()
        })))
    }
}
