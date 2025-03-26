use base64::{
    Engine as _,
    engine::{
        GeneralPurpose,
        general_purpose::{STANDARD, URL_SAFE},
    },
};

pub struct Base64;

impl mlua::UserData for Base64 {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("urlsafe", |_lua, ()| Ok(UrlSafe));
        methods.add_function("encode", |lua, val: mlua::Value| encode(lua, STANDARD, val));
        methods.add_function("decode", |lua, val: String| decode(lua, STANDARD, val));
        methods.add_meta_method(mlua::MetaMethod::Call, |lua, _this, val: mlua::Value| {
            encode(lua, STANDARD, val)
        });
    }
}

pub struct UrlSafe;

impl mlua::UserData for UrlSafe {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("encode", |lua, val: mlua::Value| encode(lua, URL_SAFE, val));
        methods.add_function("decode", |lua, val: String| decode(lua, URL_SAFE, val));
        methods.add_meta_method(mlua::MetaMethod::Call, |lua, _this, val: mlua::Value| {
            encode(lua, URL_SAFE, val)
        });
    }
}

fn encode(_lua: &mlua::Lua, engine: GeneralPurpose, val: mlua::Value) -> mlua::Result<String> {
    if let mlua::Value::String(val) = val {
        Ok(engine.encode(val.as_bytes()))
    } else {
        Err(mlua::Error::external("expected string"))
    }
}

fn decode(lua: &mlua::Lua, engine: GeneralPurpose, val: String) -> mlua::Result<mlua::Value> {
    match engine.decode(val.as_bytes()) {
        Ok(val) => Ok(mlua::Value::String(lua.create_string(&val)?)),
        Err(e) => Err(mlua::Error::external(e)),
    }
}
