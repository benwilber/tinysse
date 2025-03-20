use axum::response::sse::Event;
use mlua::LuaSerdeExt as _;
use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct Msg {
    pub id: Option<String>,
    pub event: Option<String>,
    pub data: Option<String>,
    pub comment: Option<Vec<String>>,
}

impl Msg {
    pub fn is_empty(&self) -> bool {
        self.id.is_none()
            && self.event.is_none()
            && self.data.is_none()
            && self.comment.as_ref().is_none_or(|c| c.is_empty())
    }
}

impl mlua::FromLua for Msg {
    fn from_lua(val: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
        match val.as_table() {
            Some(tbl) => {
                let mut msg = Self::default();

                if let Ok(id) = tbl.get("id") {
                    msg.id = id;
                }

                if let Ok(event) = tbl.get("event") {
                    msg.event = event;
                }

                if let Ok(data) = tbl.get("data") {
                    msg.data = data;
                }

                if let Ok(comment) = tbl.get("comment") {
                    msg.comment = comment;
                }

                Ok(msg)
            }
            None => Err(mlua::Error::FromLuaConversionError {
                from: val.type_name(),
                to: std::any::type_name::<Self>().to_string(),
                message: Some("expected table".to_owned()),
            }),
        }
    }
}

impl mlua::IntoLua for Msg {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        let tbl = lua.create_table()?;

        if let Some(id) = self.id {
            tbl.set("id", id)?;
        }

        if let Some(event) = self.event {
            tbl.set("event", event)?;
        }

        if let Some(data) = self.data {
            tbl.set("data", data)?;
        }

        if let Some(comments) = self.comment {
            if !comments.is_empty() {
                tbl.set("comment", comments)?;
            }
        }

        lua.to_value(&tbl)
    }
}

impl From<Msg> for Event {
    fn from(msg: Msg) -> Self {
        let mut event = Self::default();

        if let Some(id) = msg.id {
            event = event.id(id);
        }

        if let Some(evt) = msg.event {
            event = event.event(evt);
        }

        if let Some(data) = msg.data {
            event = event.data(data);
        }

        if let Some(comments) = msg.comment {
            for comment in comments {
                event = event.comment(comment);
            }
        }

        event
    }
}
