pub struct Mutex;

impl mlua::UserData for Mutex {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Call, |_lua, _this, ()| {
            Ok(InnerMutex {
                inner: tokio::sync::Mutex::new(()),
            })
        });
    }
}

struct InnerMutex {
    inner: tokio::sync::Mutex<()>,
}

impl mlua::UserData for InnerMutex {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_meta_method(
            mlua::MetaMethod::Call,
            |_lua, this, func: mlua::Function| async move {
                let _guard = this.inner.lock().await;
                func.call_async::<mlua::MultiValue>(()).await
            },
        );
    }
}
