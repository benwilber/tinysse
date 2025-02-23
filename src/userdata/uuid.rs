#![allow(unused_doc_comments)]

/// A Lua userdata type that provides UUID generation functionality.
///
/// This struct allows Lua scripts to generate UUIDs of different versions. It uses
/// the `uuid` crate under the hood to generate Universally Unique Identifiers (UUIDs),
/// such as UUIDv4 (randomly generated) and UUIDv7 (time-based).
///
/// # Example
/// Here's how to use the `Uuid` module in Lua:
///
/// ```lua
/// local uuid = require "uuid"
///
/// -- Generate a random UUID (v4)
/// local id1 = uuid() -- or uuid.v4()
/// print("Generated UUIDv4: " .. id1)
///
/// -- Generate a time-based UUID (v7)
/// local id2 = uuid.v7()
/// print("Generated UUIDv7: " .. id2)
/// ```
///
/// The `uuid` module can be called directly to generate a UUIDv4, and it provides
/// specific functions for generating other versions of UUIDs.
pub struct Uuid;

impl mlua::UserData for Uuid {
    /// Adds methods to the `Uuid` struct for use in Lua.
    ///
    /// This implementation registers:
    /// - A meta-method (`__call`) for generating UUIDv4 when the module is called directly.
    /// - Functions for generating specific versions of UUIDs, such as `v4` and `v7`.
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        /// Meta-method to generate a random UUID (v4).
        ///
        /// # Returns
        /// - A string representation of a randomly generated UUID (v4).
        ///
        /// # Example
        /// ```lua
        /// local uuid = require "uuid"
        /// local id = uuid()
        /// print("Generated UUIDv4: " .. id)
        /// ```
        methods.add_meta_method(mlua::MetaMethod::Call, |_lua, _this, (): ()| {
            Ok(uuid::Uuid::new_v4().to_string())
        });

        /// Function to generate a random UUID (v4).
        ///
        /// # Returns
        /// - A string representation of a randomly generated UUID (v4).
        ///
        /// # Example
        /// ```lua
        /// local uuid = require "uuid"
        /// local id = uuid.v4()
        /// print("Generated UUIDv4: " .. id)
        /// ```
        methods.add_function("v4", |_lua, ()| Ok(uuid::Uuid::new_v4().to_string()));

        /// Function to generate a time-based UUID (v7).
        ///
        /// # Returns
        /// - A string representation of a time-based UUID (v7).
        ///
        /// # Example
        /// ```lua
        /// local uuid = require "uuid"
        /// local id = uuid.v7()
        /// print("Generated UUIDv7: " .. id)
        /// ```
        methods.add_function("v7", |_lua, ()| Ok(uuid::Uuid::now_v7().to_string()));
    }
}
