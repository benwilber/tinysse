#![allow(unused_doc_comments)]
/// A Lua userdata type that provides an asynchronous sleep function.
///
/// This struct allows Lua scripts to perform a non-blocking sleep for a specified
/// duration, expressed in milliseconds. The implementation uses `tokio::time::sleep`
/// under the hood to ensure the sleep is asynchronous and does not block the event loop.
///
/// # Example
/// Here's how to use the `Sleep` struct in Lua:
///
/// ```lua
/// local sleep = require "sleep"
///
/// print("Start sleeping...")
/// sleep(1000) -- Sleep for 1000 milliseconds (1 second)
/// print("Finished sleeping!")
///
/// -- Sleep forever
/// sleep(math.huge)
/// ```
///
/// The `sleep` function can be called like a regular Lua function, and it pauses
/// execution for the specified duration.
pub struct Sleep;

impl mlua::UserData for Sleep {
    /// Adds methods to the `Sleep` struct for use in Lua.
    ///
    /// This implementation registers a meta-method for the `Sleep` struct, allowing it to
    /// be called like a function in Lua. The registered method is asynchronous and pauses
    /// execution for the specified number of milliseconds.
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        /// Registers an asynchronous meta-method for the `Sleep` struct.
        ///
        /// # MetaMethod
        /// - `__call`: Enables the `Sleep` instance to be called like a function in Lua.
        ///
        /// # Parameters
        /// - `millis` (`f64`): The number of milliseconds to sleep. Must be convertible
        ///   to a `u64` without overflow.
        ///
        /// # Returns
        /// This method does not return a value. After the specified delay, it resumes
        /// execution in Lua.
        methods.add_async_meta_method(mlua::MetaMethod::Call, async |_lua, _this, millis: f64| {
            tokio::time::sleep(std::time::Duration::from_millis(millis as u64)).await;
            Ok(())
        });
    }
}
