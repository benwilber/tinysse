use std::{fs, path::Path, time::Duration};

use crate::{
    cli::Cli,
    req::{PublishRequest, SubscribeRequest},
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
        let globals = self.lua.globals();
        globals
            .set("json", userdata::Json {})
            .expect("set userdata json");
        globals
            .set("uuid", userdata::Uuid {})
            .expect("set userdata uuid");
        globals
            .set("http", userdata::Http {})
            .expect("set userdata http");
        globals
            .set("sleep", userdata::Sleep {})
            .expect("set userdata sleep");
        globals
            .set("log", userdata::Log {})
            .expect("set userdata log");
        globals
            .set("url", userdata::Url {})
            .expect("set userdata url");
        globals
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

    pub async fn on_startup(&self, cli: &Cli) -> anyhow::Result<()> {
        let globals = self.lua.globals();

        if let Ok(startup_fn) = globals.get::<mlua::Function>("onstartup") {
            startup_fn.call_async::<()>(cli.clone()).await?;
        }

        Ok(())
    }

    pub async fn on_tick(&self, count: usize) -> anyhow::Result<()> {
        let globals = self.lua.globals();

        if let Ok(tick_fn) = globals.get::<mlua::Function>("ontick") {
            tick_fn.call_async::<()>(count).await?;
        }

        Ok(())
    }

    pub async fn on_subscribe(
        &self,
        sub_req: SubscribeRequest,
    ) -> anyhow::Result<Option<SubscribeRequest>> {
        let globals = self.lua.globals();

        if let Ok(subscribe_fn) = globals.get::<mlua::Function>("onsubscribe") {
            if let Some(sub_req) = subscribe_fn
                .call_async::<Option<SubscribeRequest>>(sub_req)
                .await?
            {
                Ok(Some(sub_req))
            } else {
                Ok(None)
            }
        } else {
            Ok(Some(sub_req))
        }
    }

    pub async fn on_publish(
        &self,
        pub_req: PublishRequest,
    ) -> anyhow::Result<Option<PublishRequest>> {
        let globals = self.lua.globals();

        if let Ok(publish_fn) = globals.get::<mlua::Function>("onpublish") {
            if let Some(pub_req) = publish_fn
                .call_async::<Option<PublishRequest>>(pub_req)
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

    pub async fn on_message(
        &self,
        pub_req: PublishRequest,
        sub_req: &SubscribeRequest,
    ) -> anyhow::Result<Option<PublishRequest>> {
        let globals = self.lua.globals();

        if let Ok(message_fn) = globals.get::<mlua::Function>("onmessage") {
            if let Some(pub_req) = message_fn
                .call_async::<Option<PublishRequest>>((pub_req, sub_req.to_owned()))
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

    pub async fn on_unsubscribe(&self, sub_req: &SubscribeRequest) -> anyhow::Result<()> {
        let globals = self.lua.globals();

        if let Ok(unsubscribe_fn) = globals.get::<mlua::Function>("onunsubscribe") {
            unsubscribe_fn.call_async::<()>(sub_req.clone()).await?;
        }

        Ok(())
    }

    pub async fn on_timeout(
        &self,
        sub_req: &SubscribeRequest,
        elapsed: &Duration,
    ) -> anyhow::Result<Option<f64>> {
        let globals = self.lua.globals();

        if let Ok(timeout_fn) = globals.get::<mlua::Function>("ontimeout") {
            return Ok(timeout_fn
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
