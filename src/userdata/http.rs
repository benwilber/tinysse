pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub struct Http;

impl Http {
    pub fn error<S: Into<String>>(msg: S) -> mlua::Error {
        mlua::Error::external(anyhow::anyhow!(msg.into()))
    }
}

impl Http {
    pub fn agent() -> Agent {
        Agent::default()
    }
}

impl mlua::UserData for Http {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("agent", |_lua, opts: Option<mlua::Table>| {
            if let Some(opts) = opts {
                Agent::new_with_opts(opts)
            } else {
                Ok(Agent::new())
            }
        });

        methods.add_async_function(
            "request",
            |lua, (method, url, opts): (String, String, Option<mlua::Table>)| async move {
                let method = reqwest::Method::from_bytes(method.as_bytes())
                    .map_err(|e| Http::error(format!("method is invalid: {e}")))?;

                Http::agent().request(&lua, method, url, opts).await
            },
        );

        methods.add_async_function(
            "get",
            |lua, (url, opts): (String, Option<mlua::Table>)| async move {
                Http::agent().get(&lua, url, opts).await
            },
        );

        methods.add_async_function(
            "head",
            |lua, (url, opts): (String, Option<mlua::Table>)| async move {
                Http::agent().head(&lua, url, opts).await
            },
        );

        methods.add_async_function(
            "post",
            |lua, (url, opts): (String, Option<mlua::Table>)| async move {
                Http::agent().post(&lua, url, opts).await
            },
        );

        methods.add_async_function(
            "put",
            |lua, (url, opts): (String, Option<mlua::Table>)| async move {
                Http::agent().put(&lua, url, opts).await
            },
        );

        methods.add_async_function(
            "patch",
            |lua, (url, opts): (String, Option<mlua::Table>)| async move {
                Http::agent().patch(&lua, url, opts).await
            },
        );

        methods.add_async_function(
            "delete",
            |lua, (url, opts): (String, Option<mlua::Table>)| async move {
                Http::agent().delete(&lua, url, opts).await
            },
        );

        methods.add_async_function(
            "options",
            |lua, (url, opts): (String, Option<mlua::Table>)| async move {
                Http::agent().options(&lua, url, opts).await
            },
        );
    }
}

pub struct Agent {
    client: reqwest::Client,
    opts: Option<mlua::Table>,
}

impl Agent {
    pub fn builder() -> reqwest::ClientBuilder {
        reqwest::Client::builder()
    }

    pub fn new() -> Self {
        Self {
            client: Self::builder()
                .user_agent(USER_AGENT)
                .build()
                .expect("build reqwest http client"),
            opts: None,
        }
    }

    pub fn new_with_opts(opts: mlua::Table) -> mlua::Result<Self> {
        let client = Self::builder()
            .user_agent(USER_AGENT)
            .build()
            .map_err(|e| Http::error(e.to_string()))?;
        Ok(Self {
            client,
            opts: Some(opts),
        })
    }

    pub async fn get<U>(
        &self,
        lua: &mlua::Lua,
        url: U,
        opts: Option<mlua::Table>,
    ) -> mlua::Result<mlua::Table>
    where
        U: AsRef<str>,
    {
        self.request(lua, http::Method::GET, url, opts).await
    }

    pub async fn head<U>(
        &self,
        lua: &mlua::Lua,
        url: U,
        opts: Option<mlua::Table>,
    ) -> mlua::Result<mlua::Table>
    where
        U: AsRef<str>,
    {
        self.request(lua, http::Method::HEAD, url, opts).await
    }

    pub async fn post<U>(
        &self,
        lua: &mlua::Lua,
        url: U,
        opts: Option<mlua::Table>,
    ) -> mlua::Result<mlua::Table>
    where
        U: AsRef<str>,
    {
        self.request(lua, http::Method::POST, url, opts).await
    }

    pub async fn put<U>(
        &self,
        lua: &mlua::Lua,
        url: U,
        opts: Option<mlua::Table>,
    ) -> mlua::Result<mlua::Table>
    where
        U: AsRef<str>,
    {
        self.request(lua, http::Method::PUT, url, opts).await
    }

    pub async fn patch<U>(
        &self,
        lua: &mlua::Lua,
        url: U,
        opts: Option<mlua::Table>,
    ) -> mlua::Result<mlua::Table>
    where
        U: AsRef<str>,
    {
        self.request(lua, http::Method::PATCH, url, opts).await
    }

    pub async fn delete<U>(
        &self,
        lua: &mlua::Lua,
        url: U,
        opts: Option<mlua::Table>,
    ) -> mlua::Result<mlua::Table>
    where
        U: AsRef<str>,
    {
        self.request(lua, http::Method::DELETE, url, opts).await
    }

    pub async fn options<U>(
        &self,
        lua: &mlua::Lua,
        url: U,
        opts: Option<mlua::Table>,
    ) -> mlua::Result<mlua::Table>
    where
        U: AsRef<str>,
    {
        self.request(lua, http::Method::OPTIONS, url, opts).await
    }

    pub async fn request<U>(
        &self,
        lua: &mlua::Lua,
        method: reqwest::Method,
        url: U,
        opts: Option<mlua::Table>,
    ) -> mlua::Result<mlua::Table>
    where
        U: AsRef<str>,
    {
        let url: url::Url = url
            .as_ref()
            .parse()
            .map_err(|e| Http::error(format!("url is invalid: {e}")))?;
        let mut req = self.client.request(method, url);

        let opts = match (self.opts.as_ref(), opts) {
            (Some(agent_opts), Some(opts)) => deep_merge(lua, agent_opts, &opts)?,
            (None, Some(opts)) => opts,
            (Some(agent_opts), None) => agent_opts.clone(),
            (None, None) => lua.create_table()?,
        };

        if let Ok(version) = opts.get::<String>("version") {
            let version = match version.to_uppercase().as_str() {
                "HTTP/0.9" => reqwest::Version::HTTP_09,
                "HTTP/1.0" => reqwest::Version::HTTP_10,
                "HTTP/1.1" => reqwest::Version::HTTP_11,
                "HTTP/2" => reqwest::Version::HTTP_2,
                "HTTP/3" => reqwest::Version::HTTP_3,
                _ => return Err(Http::error(format!("version is invalid: {version}"))),
            };
            req = req.version(version);
        }

        if let Ok(timeout) = opts.get::<f64>("timeout") {
            req = req.timeout(std::time::Duration::from_millis(timeout as u64));
        }

        if let Ok(hdrs) = opts.get::<mlua::Table>("headers") {
            for (key, val) in hdrs.pairs::<String, mlua::String>().flatten() {
                req = req.header(key, val.as_bytes().to_vec());
            }
        }

        if let Ok(args) = opts.get::<mlua::Table>("args") {
            for (key, val) in args.pairs::<String, mlua::Value>().flatten() {
                match val {
                    mlua::Value::Table(tbl) => {
                        for val in tbl.sequence_values::<String>() {
                            req = req.query(&[(&key, &val?.to_string())]);
                        }
                    }
                    _ => {
                        req = req.query(&[(key, &val.to_string()?)]);
                    }
                }
            }
        }

        if let Ok(body) = opts.get::<mlua::String>("body") {
            req = req.body(body.as_bytes().to_vec());
        }

        let res = req.send().await.map_err(|e| Http::error(e.to_string()))?;
        into_lua_res(lua, res).await
    }
}

impl Default for Agent {
    fn default() -> Self {
        Self::new()
    }
}

impl mlua::UserData for Agent {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method(
            "request",
            |lua, agent, (method, url, opts): (String, String, Option<mlua::Table>)| async move {
                let method = reqwest::Method::from_bytes(method.as_bytes())
                    .map_err(|e| Http::error(format!("method is invalid: {e}")))?;

                agent.request(&lua, method, url, opts).await
            },
        );

        methods.add_async_method(
            "get",
            |lua, agent, (url, opts): (String, Option<mlua::Table>)| async move {
                agent.get(&lua, url, opts).await
            },
        );

        methods.add_async_method(
            "head",
            |lua, agent, (url, opts): (String, Option<mlua::Table>)| async move {
                agent.head(&lua, url, opts).await
            },
        );

        methods.add_async_method(
            "post",
            |lua, agent, (url, opts): (String, Option<mlua::Table>)| async move {
                agent.post(&lua, url, opts).await
            },
        );

        methods.add_async_method(
            "put",
            |lua, agent, (url, opts): (String, Option<mlua::Table>)| async move {
                agent.put(&lua, url, opts).await
            },
        );

        methods.add_async_method(
            "patch",
            |lua, agent, (url, opts): (String, Option<mlua::Table>)| async move {
                agent.patch(&lua, url, opts).await
            },
        );

        methods.add_async_method(
            "delete",
            |lua, agent, (url, opts): (String, Option<mlua::Table>)| async move {
                agent.delete(&lua, url, opts).await
            },
        );

        methods.add_async_method(
            "options",
            |lua, agent, (url, opts): (String, Option<mlua::Table>)| async move {
                agent.options(&lua, url, opts).await
            },
        );
    }
}

async fn into_lua_res(lua: &mlua::Lua, res: reqwest::Response) -> mlua::Result<mlua::Table> {
    let tbl = lua.create_table()?;
    tbl.set("status", res.status().as_u16())?;
    tbl.set("headers", {
        let hdrs = lua.create_table()?;

        for (key, val) in res.headers() {
            hdrs.set(key.as_str(), lua.create_string(val.as_bytes())?)?;
        }

        hdrs
    })?;
    tbl.set(
        "body",
        lua.create_string(res.bytes().await.map_err(|e| Http::error(e.to_string()))?)?,
    )?;

    Ok(tbl)
}

fn deep_merge<'lua>(
    lua: &'lua mlua::Lua,
    tbl1: &'lua mlua::Table,
    tbl2: &'lua mlua::Table,
) -> mlua::Result<mlua::Table> {
    let merged = tbl1.clone();

    for (key, val) in tbl2.pairs::<mlua::Value, mlua::Value>().flatten() {
        match val {
            mlua::Value::Table(sub_tbl2) => {
                let sub_tbl1 = match tbl1.get::<mlua::Table>(&key) {
                    Ok(tbl) => tbl,
                    Err(_) => lua.create_table()?,
                };
                merged.set(key, deep_merge(lua, &sub_tbl1, &sub_tbl2)?)?;
            }
            _ => {
                merged.set(key, val)?;
            }
        }
    }

    Ok(merged)
}
