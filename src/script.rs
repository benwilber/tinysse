use std::{fs, path::Path, time::Duration};

use crate::{
    cli::Cli,
    msg::Msg,
    req::{PubReq, SubReq},
    userdata,
};

#[derive(Debug, Clone)]
pub struct Script {
    lua: mlua::Lua,
}

impl Script {
    pub fn new() -> Self {
        let script = Self {
            lua: mlua::Lua::new(),
        };
        script.init();
        script
    }

    pub fn unsafe_new() -> Self {
        let script = Self {
            lua: unsafe { mlua::Lua::unsafe_new() },
        };
        script.init();
        script
    }

    fn init(&self) {
        // Load the built-in libraries and global script
        let globals = self.lua.globals();
        let loaded = globals
            .get::<mlua::Table>("package")
            .expect("get package table")
            .get::<mlua::Table>("loaded")
            .expect("get loaded table");

        loaded
            .set("json", userdata::Json {})
            .expect("set userdata json");
        loaded
            .set("uuid", userdata::Uuid {})
            .expect("set userdata uuid");
        loaded
            .set("http", userdata::Http {})
            .expect("set userdata http");
        loaded
            .set("sleep", userdata::Sleep {})
            .expect("set userdata sleep");
        loaded
            .set("log", userdata::Log {})
            .expect("set userdata log");
        loaded
            .set("url", userdata::Url {})
            .expect("set userdata url");
        loaded
            .set("mutex", userdata::Mutex {})
            .expect("set userdata mutex");
        loaded
            .set("sqlite", userdata::Sqlite {})
            .expect("set userdata sqlite");

        self.lua
            .load(include_str!("lua/global.lua"))
            .set_name("src/lua/global.lua")
            .exec()
            .expect("load and exec src/lua/global.lua");
    }

    pub async fn load_path<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<&Self> {
        self.lua
            .load(fs::read_to_string(path.as_ref())?)
            .set_name(path.as_ref().to_string_lossy())
            .exec_async()
            .await?;
        Ok(self)
    }

    // Store the callback functions in the Lua registry for faster access
    pub fn register(&self) {
        let globals = self.lua.globals();

        for name in &[
            "tick",
            "publish",
            "subscribe",
            "catchup",
            "message",
            "unsubscribe",
            "timeout",
        ] {
            if let Ok(func) = globals.get::<mlua::Function>(*name) {
                self.lua
                    .set_named_registry_value(name, func)
                    .unwrap_or_else(|_| panic!("set registry function value {name}"));
            }
        }
    }

    pub async fn startup(&self, cli: &Cli) -> anyhow::Result<()> {
        let globals = self.lua.globals();

        if let Ok(func) = globals.get::<mlua::Function>("startup") {
            func.call_async::<()>(cli.clone()).await?;
        }

        Ok(())
    }

    pub async fn tick(&self, count: usize) -> anyhow::Result<()> {
        if let Ok(func) = self.lua.named_registry_value::<mlua::Function>("tick") {
            func.call_async::<()>(count).await?;
        }

        Ok(())
    }

    pub async fn publish(&self, pub_req: PubReq) -> anyhow::Result<Option<PubReq>> {
        if let Ok(func) = self.lua.named_registry_value::<mlua::Function>("publish") {
            if let Some(pub_req) = func.call_async::<Option<PubReq>>(pub_req).await? {
                Ok(Some(pub_req))
            } else {
                Ok(None)
            }
        } else {
            Ok(Some(pub_req))
        }
    }

    pub async fn subscribe(&self, sub_req: SubReq) -> anyhow::Result<Option<SubReq>> {
        if let Ok(func) = self.lua.named_registry_value::<mlua::Function>("subscribe") {
            if let Some(sub_req) = func.call_async::<Option<SubReq>>(sub_req).await? {
                Ok(Some(sub_req))
            } else {
                Ok(None)
            }
        } else {
            Ok(Some(sub_req))
        }
    }

    pub async fn catchup(
        &self,
        sub_req: &SubReq,
        last_event_id: &str,
    ) -> anyhow::Result<Option<Vec<Msg>>> {
        if let Ok(func) = self.lua.named_registry_value::<mlua::Function>("catchup") {
            return Ok(func
                .call_async::<Option<Vec<Msg>>>((sub_req.clone(), last_event_id))
                .await?);
        }

        Ok(None)
    }

    pub async fn message(
        &self,
        pub_req: PubReq,
        sub_req: &SubReq,
    ) -> anyhow::Result<Option<PubReq>> {
        if let Ok(func) = self.lua.named_registry_value::<mlua::Function>("message") {
            if let Some(pub_req) = func
                .call_async::<Option<PubReq>>((pub_req, sub_req.to_owned()))
                .await?
            {
                Ok(Some(pub_req))
            } else {
                Ok(None)
            }
        } else {
            Ok(Some(pub_req))
        }
    }

    pub async fn unsubscribe(&self, sub_req: &SubReq) -> anyhow::Result<()> {
        if let Ok(func) = self
            .lua
            .named_registry_value::<mlua::Function>("unsubscribe")
        {
            func.call_async::<()>(sub_req.clone()).await?;
        }

        Ok(())
    }

    pub async fn timeout(
        &self,
        sub_req: &SubReq,
        elapsed: &Duration,
    ) -> anyhow::Result<Option<f64>> {
        if let Ok(func) = self.lua.named_registry_value::<mlua::Function>("timeout") {
            return Ok(func
                .call_async::<Option<f64>>((sub_req.clone(), elapsed.as_millis()))
                .await?);
        }

        Ok(None)
    }
}

impl Default for Script {
    fn default() -> Self {
        Self::new()
    }
}
