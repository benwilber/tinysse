use std::time::Duration;

use axum::response::sse::Event;
use mlua::LuaSerdeExt as _;
use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct Message {
    pub id: Option<String>,
    pub event: Option<String>,
    pub data: Option<String>,
    #[serde(rename = "comment")]
    pub comments: Option<Vec<String>>,
    pub retry: Option<u64>,
}

impl Message {
    pub fn is_empty(&self) -> bool {
        self.id.is_none()
            && self.event.is_none()
            && self.data.is_none()
            && self.comments.as_ref().is_none_or(|c| c.is_empty())
            && self.retry.is_none()
    }
}

impl mlua::FromLua for Message {
    fn from_lua(value: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
        match value.as_table() {
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

                if let Ok(comments) = tbl.get("comments") {
                    msg.comments = comments;
                }

                if let Ok(retry) = tbl.get("retry") {
                    msg.retry = retry;
                }

                Ok(msg)
            }
            None => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "Message".to_string(),
                message: Some("expected table".to_string()),
            }),
        }
    }
}

impl mlua::IntoLua for Message {
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

        if let Some(comments) = self.comments {
            if !comments.is_empty() {
                tbl.set("comments", comments)?;
            }
        }

        if let Some(retry) = self.retry {
            tbl.set("retry", retry)?;
        }

        lua.to_value(&tbl)
    }
}

impl From<Message> for Event {
    fn from(msg: Message) -> Self {
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

        if let Some(comments) = msg.comments {
            for comment in comments {
                event = event.comment(comment);
            }
        }

        if let Some(retry) = msg.retry {
            event = event.retry(Duration::from_millis(retry));
        }

        event
    }
}
