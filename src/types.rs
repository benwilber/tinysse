use axum::response::sse;
use mlua::LuaSerdeExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: Option<String>,
    pub event: Option<String>,
    pub data: Option<String>,
    #[serde(default)]
    pub comments: Vec<String>,
    pub retry: Option<u64>,
}

impl Message {
    pub fn empty() -> Self {
        Self {
            id: None,
            event: None,
            data: None,
            comments: Vec::new(),
            retry: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.id.is_none()
            && self.event.is_none()
            && self.data.is_none()
            && self.comments.is_empty()
            && self.retry.is_none()
    }

    pub fn to_lua(&self, lua: &mlua::Lua) -> anyhow::Result<mlua::Table> {
        let table = lua.create_table()?;
        let comments = lua.create_table()?;
        comments.set_metatable(Some(lua.array_metatable()));

        for comment in &self.comments {
            comments.push(comment.clone())?;
        }

        table.set("comments", comments)?;

        if let Some(id) = &self.id {
            table.set("id", id.clone())?;
        }

        if let Some(event) = &self.event {
            table.set("event", event.clone())?;
        }

        if let Some(data) = &self.data {
            table.set("data", data.clone())?;
        }

        if let Some(retry) = self.retry {
            table.set("retry", retry)?;
        }

        Ok(table)
    }
}

impl From<Message> for sse::Event {
    fn from(msg: Message) -> Self {
        let mut evt = Self::default();

        for comment in msg.comments {
            evt = evt.comment(comment);
        }

        if let Some(id) = msg.id {
            evt = evt.id(id);
        }

        if let Some(event) = msg.event {
            evt = evt.event(event);
        }

        if let Some(data) = msg.data {
            evt = evt.data(data);
        }

        if let Some(retry) = msg.retry {
            evt = evt.retry(std::time::Duration::from_millis(retry));
        }

        evt
    }
}

impl From<mlua::Table> for Message {
    fn from(tbl: mlua::Table) -> Self {
        let mut msg = Self::empty();

        if let Ok(id) = tbl.get::<Option<String>>("id") {
            msg.id = id;
        }

        if let Ok(event) = tbl.get::<Option<String>>("event") {
            msg.event = event;
        }

        if let Ok(data) = tbl.get::<Option<String>>("data") {
            msg.data = data;
        }

        if let Ok(retry) = tbl.get::<Option<u64>>("retry") {
            msg.retry = retry;
        }

        if let Ok(cmts) = tbl.get::<mlua::Table>("comment") {
            for comment in cmts.sequence_values::<String>().flatten() {
                msg.comments.push(comment);
            }
        }

        msg
    }
}
