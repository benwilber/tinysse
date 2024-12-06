use std::{fs, path::Path};

use crate::{types::Message, userdata};
use mlua::Lua;

#[derive(Debug, Clone)]
pub struct Script {
    lua: Lua,
}

impl Script {
    pub fn new() -> Self {
        let script = Self { lua: Lua::new() };
        script.init();
        script
    }

    pub fn unsafe_new() -> Self {
        let script = Self {
            lua: unsafe { Lua::unsafe_new() },
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

    pub async fn publish(&self, msg: &Message) -> anyhow::Result<Option<Message>> {
        let globals = self.lua.globals();

        if let Ok(publish_fn) = globals.get::<mlua::Function>("publish") {
            let lua_msg = msg.to_lua(&self.lua)?;

            if let Some(msg) = publish_fn
                .call_async::<Option<mlua::Table>>(lua_msg)
                .await?
            {
                Ok(Some(msg.into()))
            } else {
                Ok(None)
            }
        } else {
            Ok(Some(msg.to_owned()))
        }
    }

    pub async fn message(&self, msg: &Message) -> anyhow::Result<Option<Message>> {
        let globals = self.lua.globals();

        if let Ok(message_fn) = globals.get::<mlua::Function>("message") {
            let lua_msg = msg.to_lua(&self.lua)?;

            if let Some(msg) = message_fn
                .call_async::<Option<mlua::Table>>(lua_msg)
                .await?
            {
                Ok(Some(msg.into()))
            } else {
                Ok(None)
            }
        } else {
            Ok(Some(msg.to_owned()))
        }
    }
}

impl Default for Script {
    fn default() -> Self {
        Self::new()
    }
}
