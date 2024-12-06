use mlua::LuaSerdeExt as _;
use std::{collections::HashMap, net::SocketAddr};

use crate::msg::Message;

#[derive(Debug, Clone)]
pub struct Addr {
    ip: String,
    port: u16,
}

impl From<SocketAddr> for Addr {
    fn from(addr: SocketAddr) -> Self {
        Addr {
            ip: addr.ip().to_string(),
            port: addr.port(),
        }
    }
}

impl mlua::FromLua for Addr {
    fn from_lua(value: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
        match value.as_table() {
            Some(tbl) => Ok(Addr {
                ip: tbl.get("ip")?,
                port: tbl.get("port")?,
            }),
            None => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "Addr".to_string(),
                message: Some("expected table".to_string()),
            }),
        }
    }
}

impl mlua::IntoLua for Addr {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        let tbl = lua.create_table()?;

        tbl.set("ip", self.ip)?;
        tbl.set("port", self.port)?;

        lua.to_value(&tbl)
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    addr: Addr,
    method: String,
    uri: String,
    headers: HashMap<String, String>,
}

impl Request {
    pub fn new(addr: SocketAddr, req: &axum::extract::Request) -> Self {
        Request {
            addr: addr.into(),
            method: req.method().to_string(),
            uri: req.uri().to_string(),
            headers: req
                .headers()
                .iter()
                .filter_map(|(k, v)| {
                    if let Ok(v) = v.to_str() {
                        Some((k.as_str().to_string(), v.to_string()))
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }

    pub fn addr(&self) -> &Addr {
        &self.addr
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }
}

impl mlua::FromLua for Request {
    fn from_lua(value: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
        match value.as_table() {
            Some(tbl) => Ok(Request {
                addr: tbl.get("addr")?,
                method: tbl.get("method")?,
                uri: tbl.get("uri")?,
                headers: tbl.get("headers")?,
            }),
            None => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "Request".to_string(),
                message: Some("expected table".to_string()),
            }),
        }
    }
}

impl mlua::IntoLua for Request {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        let tbl = lua.create_table()?;

        tbl.set("addr", self.addr)?;
        tbl.set("method", self.method)?;
        tbl.set("uri", self.uri)?;
        tbl.set("headers", self.headers)?;

        lua.to_value(&tbl)
    }
}

#[derive(Debug, Clone)]
pub struct PublishRequest {
    req: Request,
    msg: Message,
    meta: Option<mlua::Table>,
}

impl PublishRequest {
    pub fn new(req: Request, msg: Message) -> Self {
        Self {
            req,
            msg,
            meta: None,
        }
    }

    pub fn req(&self) -> &Request {
        &self.req
    }

    pub fn msg(&self) -> &Message {
        &self.msg
    }

    pub fn meta(&self) -> Option<&mlua::Table> {
        self.meta.as_ref()
    }
}

impl mlua::FromLua for PublishRequest {
    fn from_lua(value: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
        match value.as_table() {
            Some(tbl) => {
                let req = tbl.get("req")?;
                tbl.set("req", mlua::Value::Nil)?;

                let msg = tbl.get("msg")?;
                tbl.set("msg", mlua::Value::Nil)?;

                Ok(Self {
                    req,
                    msg,
                    meta: Some(tbl.clone()),
                })
            }
            None => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "Publish".to_string(),
                message: Some("expected table".to_string()),
            }),
        }
    }
}

impl mlua::IntoLua for PublishRequest {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        let tbl = match self.meta {
            Some(tbl) => tbl,
            None => lua.create_table()?,
        };

        tbl.set("req", self.req)?;
        tbl.set("msg", self.msg)?;

        lua.to_value(&tbl)
    }
}

#[derive(Debug, Clone)]
pub struct SubscribeRequest {
    req: Request,
    meta: Option<mlua::Table>,
}

impl SubscribeRequest {
    pub fn new(req: Request) -> Self {
        Self { req, meta: None }
    }

    pub fn req(&self) -> &Request {
        &self.req
    }

    pub fn meta(&self) -> Option<&mlua::Table> {
        self.meta.as_ref()
    }
}

impl mlua::FromLua for SubscribeRequest {
    fn from_lua(value: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
        match value.as_table() {
            Some(tbl) => {
                let req = tbl.get("req")?;
                tbl.set("req", mlua::Value::Nil)?;

                Ok(Self {
                    req,
                    meta: Some(tbl.clone()),
                })
            }
            None => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "Subscribe".to_string(),
                message: Some("expected table".to_string()),
            }),
        }
    }
}

impl mlua::IntoLua for SubscribeRequest {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        let tbl = match self.meta {
            Some(tbl) => tbl,
            None => lua.create_table()?,
        };

        tbl.set("req", self.req)?;

        lua.to_value(&tbl)
    }
}
