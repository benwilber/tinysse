pub struct Fernet;

impl mlua::UserData for Fernet {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("genkey", |_, ()| Ok(fernet::Fernet::generate_key()));
        methods.add_meta_method(
            mlua::MetaMethod::Call,
            |_lua, _this, key: Option<String>| {
                let key = key.unwrap_or_else(fernet::Fernet::generate_key);

                Ok(InnerFernet {
                    inner: match fernet::Fernet::new(&key) {
                        Some(fernet) => fernet,
                        None => {
                            return Err(mlua::Error::external(
                                "key must be 32-bytes, url-safe base64-encoded",
                            ));
                        }
                    },
                })
            },
        );
    }
}

struct InnerFernet {
    inner: fernet::Fernet,
}

impl mlua::UserData for InnerFernet {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("encrypt", |_lua, this, val: mlua::Value| {
            if let mlua::Value::String(val) = val {
                Ok(this.inner.encrypt(&val.as_bytes()))
            } else {
                Err(mlua::Error::FromLuaConversionError {
                    from: val.type_name(),
                    to: "string".to_owned(),
                    message: Some("expected string".to_owned()),
                })
            }
        });

        methods.add_method("decrypt", |lua, this, (val, ttl): (String, Option<u64>)| {
            let plain = if let Some(ttl) = ttl {
                this.inner.decrypt_with_ttl(&val, ttl)
            } else {
                this.inner.decrypt(&val)
            };

            match plain {
                Ok(plain) => Ok(Some(lua.create_string(&plain)?)),
                Err(_) => Ok(None),
            }
        });
    }
}
