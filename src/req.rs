use mlua::LuaSerdeExt as _;
use std::{collections::HashMap, net::SocketAddr};

use crate::{msg::Msg, state::AppState};

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
    fn from_lua(val: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
        match val.as_table() {
            Some(tbl) => Ok(Addr {
                ip: tbl.get("ip")?,
                port: tbl.get("port")?,
            }),
            None => Err(mlua::Error::FromLuaConversionError {
                from: val.type_name(),
                to: std::any::type_name::<Self>().to_string(),
                message: Some("expected table".to_owned()),
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
pub struct Req {
    addr: Addr,
    method: String,
    uri: String,
    path: String,
    query: String,
    headers: HashMap<String, String>,
}

impl Req {
    pub fn new(addr: SocketAddr, req: &axum::extract::Request) -> Self {
        Req {
            addr: addr.into(),
            method: req.method().to_string(),
            uri: req.uri().to_string(),
            path: req.uri().path().to_string(),
            query: req.uri().query().map(String::from).unwrap_or_default(),
            headers: req
                .headers()
                .iter()
                .filter_map(|(k, v)| {
                    // Only include headers that are valid UTF-8
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

impl mlua::FromLua for Req {
    fn from_lua(val: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
        match val.as_table() {
            Some(tbl) => Ok(Req {
                addr: tbl.get("addr")?,
                method: tbl.get("method")?,
                uri: tbl.get("uri")?,
                path: tbl.get("path")?,
                query: tbl.get("query").unwrap_or_default(),
                headers: tbl.get("headers")?,
            }),
            None => Err(mlua::Error::FromLuaConversionError {
                from: val.type_name(),
                to: std::any::type_name::<Self>().to_string(),
                message: Some("expected table".to_string()),
            }),
        }
    }
}

impl mlua::IntoLua for Req {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        let tbl = lua.create_table()?;

        tbl.set("addr", self.addr)?;
        tbl.set("method", self.method)?;
        tbl.set("uri", self.uri)?;
        tbl.set("path", self.path)?;
        tbl.set("query", self.query)?;
        tbl.set("headers", self.headers)?;

        lua.to_value(&tbl)
    }
}

#[derive(Debug, Clone)]
pub struct PubReq {
    req: Req,
    msg: Msg,
    meta: Option<mlua::Table>,
}

impl PubReq {
    pub fn new(req: Req, msg: Msg) -> Self {
        Self {
            req,
            msg,
            meta: None,
        }
    }

    pub fn req(&self) -> &Req {
        &self.req
    }

    pub fn msg(&self) -> &Msg {
        &self.msg
    }

    pub fn meta(&self) -> Option<&mlua::Table> {
        self.meta.as_ref()
    }
}

impl mlua::FromLua for PubReq {
    fn from_lua(val: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
        match val.as_table() {
            Some(tbl) => {
                let req = tbl.get("req")?;
                tbl.set("req", mlua::Value::Nil)?;

                let msg = tbl.get("msg")?;
                tbl.set("msg", mlua::Value::Nil)?;

                Ok(Self {
                    req,
                    msg,
                    meta: Some(tbl.to_owned()),
                })
            }
            None => Err(mlua::Error::FromLuaConversionError {
                from: val.type_name(),
                to: std::any::type_name::<Self>().to_string(),
                message: Some("expected table".to_string()),
            }),
        }
    }
}

impl mlua::IntoLua for PubReq {
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
pub struct SubReq {
    req: Req,
    last_event_id: Option<String>,
    meta: Option<mlua::Table>,
}

impl SubReq {
    pub fn new(req: Req, last_event_id: Option<String>) -> Self {
        Self {
            req,
            last_event_id,
            meta: None,
        }
    }

    pub fn req(&self) -> &Req {
        &self.req
    }

    pub fn last_event_id(&self) -> Option<&str> {
        self.last_event_id.as_deref()
    }

    pub fn meta(&self) -> Option<&mlua::Table> {
        self.meta.as_ref()
    }
}

impl mlua::FromLua for SubReq {
    fn from_lua(val: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
        match val.as_table() {
            Some(tbl) => {
                let req = tbl.get("req")?;
                tbl.set("req", mlua::Value::Nil)?;

                let last_event_id = tbl.get("last_event_id")?;
                tbl.set("last_event_id", mlua::Value::Nil)?;

                Ok(Self {
                    req,
                    last_event_id,
                    meta: Some(tbl.to_owned()),
                })
            }
            None => Err(mlua::Error::FromLuaConversionError {
                from: val.type_name(),
                to: std::any::type_name::<Self>().to_string(),
                message: Some("expected table".to_owned()),
            }),
        }
    }
}

impl mlua::IntoLua for SubReq {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        let tbl = match self.meta {
            Some(tbl) => tbl,
            None => lua.create_table()?,
        };

        tbl.set("req", self.req)?;
        tbl.set("last_event_id", self.last_event_id)?;

        lua.to_value(&tbl)
    }
}

pub struct SubReqGuard<'a> {
    state: &'a AppState,
    sub_req: SubReq,
}

impl<'a> SubReqGuard<'a> {
    pub fn new(state: &'a AppState, sub_req: SubReq) -> Self {
        Self { state, sub_req }
    }
}

impl Drop for SubReqGuard<'_> {
    fn drop(&mut self) {
        let state = self.state.clone();
        let sub_req = self.sub_req.clone();

        tokio::spawn(async move {
            if let Err(e) = state.script.unsubscribe(&sub_req).await {
                tracing::error!("{e}");
            }
        });
    }
}
