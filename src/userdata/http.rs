#![allow(unused_doc_comments)]
/// A Lua userdata type that provides HTTP request functionality.
///
/// This struct enables Lua scripts to perform HTTP requests using the `reqwest` crate. It supports
/// configurable options such as method, URL, headers, query parameters, body, and timeout. The response
/// is returned as a Lua table containing the status, headers, and body of the HTTP response.
///
/// # Example
/// Here's how to use the `Http` module in Lua:
///
/// ```lua
/// local http = require "http"
///
/// -- Perform an HTTP GET request
/// local r = http.request(
///     "GET",
///     "http://httpbin.org/get",
///     {
///         query = { key = "value" } -- Appends ?key=value to the URL
///         headers = { ["Accept"] = "application/json" }
///     }
/// )
///
/// print("Status:", r.status)
/// print("Headers:", r.headers)
/// print("Body:", r.body)
///
/// -- Perform an HTTP POST request with a body
/// local r = http.request(
///     "POST",
///     "https://httpbin.org/post",
///     {
///         headers = { ["Content-Type"] = "application/json" },
///         body = '{"key": "value"}'
///     }
/// )
/// print("Status:", r.status)
/// print("Body:", r.body)
/// ```
///
/// The `http.request` method is asynchronous and returns a table with the response details.
///
/// The standard HTTP methods are supported: `GET`, `HEAD`, `OPTIONS`, `POST`, `PUT`, `PATCH`, and `DELETE`.
pub struct Http;

impl Http {
    /// Creates an external Lua error with the provided message.
    ///
    /// # Parameters
    /// - `msg` (`S: Into<String>`): The error message to include.
    ///
    /// # Returns
    /// - `mlua::Error`: A Lua error object with the provided message.
    fn error<S: Into<String>>(msg: S) -> mlua::Error {
        mlua::Error::external(anyhow::anyhow!(msg.into()))
    }

    pub async fn request(
        lua: &mlua::Lua,
        method: &str,
        url: &str,
        options: &Option<mlua::Table>,
    ) -> mlua::Result<mlua::Table> {
        let method: reqwest::Method = method
            .parse()
            .map_err(|_| Self::error("method is invalid"))?;
        let url: url::Url = url.parse().map_err(|_| Self::error("url is invalid"))?;

        let mut client = reqwest::Client::default().request(method, url);

        if let Some(options) = options {
            if let Ok(version) = options.get::<String>("version") {
                let version = match version.to_uppercase().as_str() {
                    "HTTP/0.9" => reqwest::Version::HTTP_09,
                    "HTTP/1.0" => reqwest::Version::HTTP_10,
                    "HTTP/1.1" => reqwest::Version::HTTP_11,
                    "HTTP/2" => reqwest::Version::HTTP_2,
                    "HTTP/3" => reqwest::Version::HTTP_3,
                    _ => return Err(Self::error("version is invalid")),
                };
                client = client.version(version);
            }

            if let Ok(timeout) = options.get::<f64>("timeout") {
                client = client.timeout(std::time::Duration::from_millis(timeout as u64));
            }

            if let Ok(headers) = options.get::<mlua::Table>("headers") {
                for (key, value) in headers.pairs::<String, String>().flatten() {
                    client = client.header(key, value);
                }
            }

            if let Ok(query) = options.get::<mlua::Table>("query") {
                client = client.query(&query);
            }

            if let Ok(body) = options.get::<String>("body") {
                client = client.body(body);
            }
        }

        let resp = client.send().await.map_err(mlua::Error::external)?;
        to_lua_resp(lua, resp).await
    }
}

impl mlua::UserData for Http {
    /// Adds methods to the `Http` struct for use in Lua.
    ///
    /// Methods include:
    /// - `http.request(options)`: Performs an HTTP request and returns the response as a Lua table.
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        /// Performs an HTTP request.
        ///
        /// # Parameters
        /// The `options` table must include:
        /// - `method` (`string`): The HTTP method (e.g., "GET", "POST", "PUT").
        /// - `url` (`string`): The target URL.
        ///
        /// Optional fields:
        /// - `headers` (`table<string, string>`): A table of HTTP headers.
        /// - `query` (`table<string, string>`): A table of query parameters.
        /// - `body` (`string`): The request body.
        /// - `timeout` (`number`): Timeout in milliseconds.
        ///
        /// # Returns
        /// A Lua table with the following fields:
        /// - `status` (`number`): The HTTP status code.
        /// - `headers` (`table<string, string>`): A table of response headers.
        /// - `body` (`string`): The response body.
        ///
        /// # Example
        /// ```lua
        /// local response = http.request({
        ///     method = "GET",
        ///     url = "https://example.com"
        /// })
        /// print("Status:", response.status)
        /// print("Body:", response.body)
        /// ```
        methods.add_async_function(
            "request",
            |lua, (method, url, options): (String, String, Option<mlua::Table>)| async move {
                Self::request(&lua, &method, &url, &options).await
            },
        );

        methods.add_async_function(
            "get",
            |lua, (url, options): (String, Option<mlua::Table>)| async move {
                Self::request(&lua, reqwest::Method::GET.as_str(), &url, &options).await
            },
        );

        methods.add_async_function(
            "head",
            |lua, (url, options): (String, Option<mlua::Table>)| async move {
                Self::request(&lua, reqwest::Method::HEAD.as_str(), &url, &options).await
            },
        );

        methods.add_async_function(
            "options",
            |lua, (url, options): (String, Option<mlua::Table>)| async move {
                Self::request(&lua, reqwest::Method::OPTIONS.as_str(), &url, &options).await
            },
        );

        methods.add_async_function(
            "post",
            |lua, (url, options): (String, Option<mlua::Table>)| async move {
                Self::request(&lua, reqwest::Method::POST.as_str(), &url, &options).await
            },
        );

        methods.add_async_function(
            "put",
            |lua, (url, options): (String, Option<mlua::Table>)| async move {
                Self::request(&lua, reqwest::Method::PUT.as_str(), &url, &options).await
            },
        );

        methods.add_async_function(
            "patch",
            |lua, (url, options): (String, Option<mlua::Table>)| async move {
                Self::request(&lua, reqwest::Method::PATCH.as_str(), &url, &options).await
            },
        );
        methods.add_async_function(
            "delete",
            |lua, (url, options): (String, Option<mlua::Table>)| async move {
                Self::request(&lua, reqwest::Method::DELETE.as_str(), &url, &options).await
            },
        );
    }
}

/// Converts an HTTP response into a Lua table.
///
/// # Parameters
/// - `lua` (`&mlua::Lua`): The Lua context.
/// - `resp` (`reqwest::Response`): The HTTP response to convert.
///
/// # Returns
/// - `Ok(mlua::Table)`: A Lua table containing the response details.
/// - `Err(mlua::Error)`: If an error occurs while processing the response.
async fn to_lua_resp(lua: &mlua::Lua, resp: reqwest::Response) -> mlua::Result<mlua::Table> {
    let table = lua.create_table()?;
    table.set("status", resp.status().as_u16())?;
    let headers = lua.create_table()?;

    for (key, value) in resp.headers() {
        headers.set(key.as_str(), value.to_str().map_err(mlua::Error::external)?)?;
    }

    table.set("headers", headers)?;
    table.set("body", resp.text().await.map_err(mlua::Error::external)?)?;
    Ok(table)
}
