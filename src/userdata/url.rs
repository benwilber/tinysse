use std::collections::HashMap;

/// A Lua userdata type that provides URL manipulation functionality.
///
/// This struct allows Lua scripts to encode, decode, and manipulate URLs. It supports
/// constructing URLs from parts, extracting components from a URL string, and encoding
/// or decoding query parameters using HTML form-style encoding.
///
/// # Example
/// Here's how to use the `Url` module in Lua:
///
/// ```lua
/// local url = require "url"
///
/// -- Encode a URL from parts
/// url.encode {
///   scheme = "https",
///   username = "user",
///   password = "pass",
///   host = "example.com",
///   port = 443,
///   query = "key=value",
///   fragment = "section",
///   args = {
///     key1 = {
///         "value1",
///         "value2"
///     },
///     key2 = {
///         "value3"
///     }
///   }
/// }
/// -- "https://user:pass@example.com:443/?key=value&key1=value1&key1=value2&key2=value3#section"
/// --
/// -- Note that `query` and `args` are merged if both are provided.
///
/// -- Decode a URL into parts
/// url.decode "https://user:pass@example.com:443/path?key=value#section"
/// -- {
/// --   scheme = "https",
/// --   username = "user",
/// --   password = "pass",
/// --   host = "example.com",
/// --   query = "key=value",
/// --   fragment = "section",
/// --   args = {
/// --     key = {
/// --       "value"
/// --     }
/// --   }
/// -- }
///
/// -- Quote (serialize) a Lua table into query parameters (`application/x-www-form-urlencoded`)
/// url.quote {
///   key = {
///     "value1",
///     "value2"
///   },
///   other = {
///     "value3"
///   }
/// }
/// -- "key=value1&key=value2&other=value3"
///
/// -- Unquote (deserialize) query parameters into a Lua table
/// url.unquote "key=value1&key=value2&other=value3"
/// -- {
/// --   key = {
/// --     "value1",
/// --     "value2"
/// --   },
/// --   other = {
/// --     "value3"
/// --   }
/// -- }
/// ```
pub struct Url;

impl Url {
    /// Encodes a URL from its components.
    ///
    /// # Parameters
    /// - `parts` (`&mlua::Table`): A Lua table containing URL components. The following fields are supported:
    ///   - `scheme` (`string`): The URL scheme (e.g., "http", "https"). **Required**.
    ///   - `host` (`string`): The hostname or IP address. **Required**.
    ///   - `username` (`string`): The username for authentication. Optional.
    ///   - `password` (`string`): The password for authentication. Optional.
    ///   - `port` (`number`): The port number. Optional.
    ///   - `path` (`string`): The URL path. Optional, defaults to "/".
    ///   - `query` (`string`): The query string. Optional.
    ///   - `args` (`table<string, table<string>>`): Query parameters as key-value pairs. Each value is an array of strings.
    ///   - `fragment` (`string`): The URL fragment. Optional.
    ///
    /// # Returns
    /// - `Ok(String)`: The constructed URL string.
    /// - `Err(mlua::Error)`: If required fields are missing or invalid.
    pub fn encode(parts: &mlua::Table) -> Result<String, mlua::Error> {
        let mut url = url::Url::parse("http://example.com").expect("parse placeholder url");

        if let Ok(scheme) = parts.get::<String>("scheme") {
            url.set_scheme(&scheme)
                .map_err(|_| mlua::Error::external(anyhow::anyhow!("scheme is invalid")))?;
        } else {
            return Err(mlua::Error::external(anyhow::anyhow!("scheme is required")));
        }

        if let Ok(username) = parts.get::<String>("username") {
            url.set_username(&username)
                .map_err(|_| mlua::Error::external(anyhow::anyhow!("username is invalid")))?;
        }

        if let Ok(password) = parts.get::<String>("password") {
            url.set_password(Some(&password))
                .map_err(|_| mlua::Error::external(anyhow::anyhow!("password is invalid")))?;
        }

        if let Ok(host) = parts.get::<String>("host") {
            url.set_host(Some(&host))
                .map_err(|_| mlua::Error::external(anyhow::anyhow!("host is invalid")))?;
        } else {
            return Err(mlua::Error::external(anyhow::anyhow!("host is required")));
        }

        if let Ok(port) = parts.get::<u16>("port") {
            url.set_port(Some(port))
                .map_err(|_| mlua::Error::external(anyhow::anyhow!("port is invalid")))?;
        }

        if let Ok(path) = parts.get::<String>("path") {
            url.set_path(&path);
        }

        if let Ok(query) = parts.get::<String>("query") {
            url.set_query(Some(&query));
        }

        if let Ok(args) = parts.get::<HashMap<String, Vec<String>>>("args") {
            for (key, vals) in args {
                for val in vals {
                    url.query_pairs_mut().append_pair(&key, &val);
                }
            }
        }

        if let Ok(fragment) = parts.get::<String>("fragment") {
            url.set_fragment(Some(&fragment));
        }

        Ok(url.to_string())
    }

    /// Decodes a URL string into its components.
    ///
    /// # Parameters
    /// - `lua` (`&mlua::Lua`): The Lua context.
    /// - `value` (`&str`): The URL string to decode.
    ///
    /// # Returns
    /// - `Ok(mlua::Table)`: A Lua table containing the URL components:
    ///   - `scheme` (`string`): The URL scheme.
    ///   - `host` (`string`): The hostname or IP address.
    ///   - `username` (`string`): The username.
    ///   - `password` (`string`): The password.
    ///   - `port` (`number`): The port number.
    ///   - `query` (`string`): The raw query string.
    ///   - `args` (`table<string, table<string>>`): Parsed query parameters as key-value pairs.
    ///   - `fragment` (`string`): The URL fragment.
    /// - `Err(mlua::Error)`: If the URL string is invalid.
    pub fn decode(lua: &mlua::Lua, value: &str) -> Result<mlua::Table, mlua::Error> {
        let url = url::Url::parse(value).map_err(mlua::Error::external)?;
        let table = lua.create_table()?;

        let mut args: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for (key, value) in url.query_pairs() {
            args.entry(key.to_string())
                .or_default()
                .push(value.to_string());
        }
        table.set("args", args).ok();

        table.set("scheme", url.scheme()).ok();
        table.set("username", url.username()).ok();
        table.set("password", url.password()).ok();
        table.set("host", url.host().map(|h| h.to_string())).ok();
        table.set("port", url.port()).ok();
        table.set("query", url.query()).ok();
        table.set("fragment", url.fragment()).ok();

        Ok(table)
    }

    /// Serializes a Lua table into a query string.
    ///
    /// # Parameters
    /// - `value` (`&mlua::Table`): A Lua table representing query parameters.
    ///
    /// # Returns
    /// - `Ok(String)`: The serialized query string.
    /// - `Err(mlua::Error)`: If serialization fails.
    pub fn quote(value: &mlua::Table) -> Result<String, mlua::Error> {
        serde_html_form::to_string(value).map_err(mlua::Error::external)
    }

    /// Deserializes a query string into a Lua table.
    ///
    /// # Parameters
    /// - `lua` (`&mlua::Lua`): The Lua context.
    /// - `value` (`&str`): The query string to deserialize.
    ///
    /// # Returns
    /// - `Ok(mlua::Table)`: A Lua table containing query parameters as key-value pairs.
    /// - `Err(mlua::Error)`: If deserialization fails.
    pub fn unquote(lua: &mlua::Lua, value: &str) -> Result<mlua::Table, mlua::Error> {
        let form: std::collections::HashMap<String, Vec<String>> =
            serde_html_form::from_str(value).map_err(mlua::Error::external)?;
        lua.create_table_from(form)
    }
}

impl mlua::UserData for Url {
    /// Adds methods to the `Url` struct for use in Lua.
    ///
    /// Methods include:
    /// - `url(parts)`: Encodes a URL from components. (Shorthand for `url.encode`).
    /// - `url.encode(parts)`: Encodes a URL from components.
    /// - `url.decode(url_string)`: Decodes a URL into components.
    /// - `url.quote(table)`: Serializes a table into a query string.
    /// - `url.unquote(query_string)`: Deserializes a query string into a table.
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Call, |_lua, _this, parts: mlua::Table| {
            Self::encode(&parts)
        });
        methods.add_function("encode", |_lua, parts: mlua::Table| Self::encode(&parts));
        methods.add_function("decode", |lua, value: String| Self::decode(lua, &value));
        methods.add_function("quote", |_lua, value: mlua::Table| Self::quote(&value));
        methods.add_function("unquote", |lua, value: String| Self::unquote(lua, &value));
    }
}
