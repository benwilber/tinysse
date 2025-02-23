#![allow(unused_doc_comments)]
use mlua::LuaSerdeExt as _;
/// A Lua userdata type that provides JSON encoding and decoding functionality.
///
/// This struct allows Lua scripts to encode, decode, and work with JSON data. It provides
/// functions for serializing Lua values into JSON strings, deserializing JSON strings into
/// Lua values, and creating JSON-like arrays. Additionally, it includes utility methods
/// for pretty-printing JSON.
///
/// # Example
/// Here's how to use the `Json` module in Lua:
///
/// ```lua
/// local json = require "json"
///
/// -- Encode a Lua table into a JSON string
/// json.encode { key = "value", arr = { 1, 2, 3 } }
/// -- '{"key": "value", "arr": [1, 2, 3]}'
///
/// -- Decode a JSON string into a Lua table
/// json.decode '{"key": "value", "arr": [1, 2, 3]}'
/// -- { key = "value", arr = { 1, 2, 3 } }
///
/// -- Create a JSON array
/// json.array()
/// -- '[]'
/// json.array { 1, 2, 3 }
/// -- '[1, 2, 3]'
///
/// -- Use `null` in JSON
/// json { null = json.null }
/// -- '{"null": null}'
///
/// -- Print a Lua table as JSON
/// -- (Shorthand for `print(json.encode(value))`)
/// json.print { nested = { key = "value" , null = json.null, arr = { 1, 2, 3 } } }
/// -- {"nested":{"key":"value","arr":[1,2,3],"null":null}}
///
/// -- Pretty-print a Lua table as JSON
/// -- (Shorthand for `print(json.encode(value, true))`)
/// json.pprint { nested = { key = "value" , null = json.null, arr = { 1, 2, 3 } } }
/// -- {
/// --   "nested": {
/// --     "arr": [
/// --       1,
/// --       2,
/// --       3
/// --     ],
/// --     "key": "value",
/// --     "null": null
/// --   }
/// -- }
/// ```
///
/// The `json` module can be called directly to encode values, and it provides
/// specific methods for other operations.
pub struct Json;

impl Json {
    /// Encodes a Lua value into a JSON string.
    ///
    /// # Parameters
    /// - `value` (`&mlua::Value`): The Lua value to encode.
    /// - `pretty` (`Option<bool>`): If `Some(true)`, the JSON string will be pretty-printed.
    ///
    /// # Returns
    /// - `Ok(Some(String))`: The JSON string representation of the value.
    /// - `Err(mlua::Error)`: If the value cannot be serialized.
    fn encode(value: &mlua::Value, pretty: Option<bool>) -> Result<Option<String>, mlua::Error> {
        match pretty {
            Some(true) => match serde_json::to_string_pretty(&value) {
                Ok(s) => Ok(Some(s)),
                Err(e) => Err(mlua::Error::SerializeError(e.to_string())),
            },
            _ => match serde_json::to_string(&value) {
                Ok(s) => Ok(Some(s)),
                Err(e) => Err(mlua::Error::SerializeError(e.to_string())),
            },
        }
    }
}

impl mlua::UserData for Json {
    /// Adds fields to the `Json` struct for use in Lua.
    ///
    /// Includes:
    /// - `json.null`: Represents the JSON `null` value in Lua.
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("null", |lua, _this| Ok(lua.null()));
    }

    /// Adds functions to the `Json` struct for use in Lua.
    ///
    /// Functions include:
    /// - `json.encode(value, ?pretty)`: Encodes a Lua value as a JSON string.
    /// - `json.decode(str)`: Decodes a JSON string into a Lua value.
    /// - `json.array(table)`: Creates a JSON-like array.
    /// - `json.print(value)`: Prints a JSON string representation of the value.
    /// - `json.pprint(value)`: Pretty-prints a JSON string representation of the value.
    /// - `json()`: Shortcut for encoding a value into JSON.
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        /// Creates a JSON-like array from a Lua table.
        ///
        /// # Parameters
        /// - `table` (`Option<mlua::Table>`): A Lua table to convert into a JSON array. If `nil`, an empty array is created.
        ///
        /// # Returns
        /// - `Ok(mlua::Table)`: A Lua table with JSON array behavior.
        methods.add_function("array", |lua, table: Option<mlua::Table>| {
            let array = match table {
                Some(table) => table,
                None => lua.create_table()?,
            };

            array.set_metatable(Some(lua.array_metatable()));

            Ok(array)
        });

        /// Encodes a Lua value as a JSON string.
        ///
        /// # Parameters
        /// - `value` (`mlua::Value`): The Lua value to encode.
        /// - `pretty` (`Option<bool>`): If `Some(true)`, the JSON string will be pretty-printed.
        ///
        /// # Returns
        /// - `Ok(String)`: The JSON string representation of the value.
        /// - `Err(mlua::Error)`: If the value cannot be encoded as JSON.
        methods.add_function(
            "encode",
            |_lua, (value, pretty): (mlua::Value, Option<bool>)| Self::encode(&value, pretty),
        );

        /// Meta-method to encode a Lua value as a JSON string when the module is called.
        ///
        /// # Example
        /// ```lua
        /// local json = require "json"
        /// local str = json { key = "value" }
        /// print(str)
        /// ```
        methods.add_meta_method(
            mlua::MetaMethod::Call,
            |_lua, _json, (value, pretty): (mlua::Value, Option<bool>)| {
                Self::encode(&value, pretty)
            },
        );

        /// Decodes a JSON string into a Lua value.
        ///
        /// # Parameters
        /// - `value` (`String`): The JSON string to decode.
        ///
        /// # Returns
        /// - `Ok(mlua::Value)`: The Lua representation of the JSON data.
        /// - `Err(mlua::Error)`: If the JSON string cannot be decoded.
        methods.add_function("decode", |lua, value: String| {
            match serde_json::from_str::<serde_json::Value>(&value) {
                Ok(value) => Ok(lua
                    .to_value(&value)
                    .map_err(|e| mlua::Error::DeserializeError(e.to_string()))),
                Err(e) => Err(mlua::Error::DeserializeError(e.to_string())),
            }
        });

        /// Prints a JSON string representation of the Lua value.
        ///
        /// # Parameters
        /// - `value` (`mlua::Value`): The Lua value to print.
        methods.add_function("print", |lua, value: mlua::Value| {
            let globals = lua.globals();
            let print = globals.get::<mlua::Function>("print")?;
            print.call::<()>(Self::encode(&value, Some(false))?)?;
            Ok(())
        });

        /// Pretty-prints a JSON string representation of the Lua value.
        ///
        /// # Parameters
        /// - `value` (`mlua::Value`): The Lua value to pretty-print.
        methods.add_function("pprint", |lua, value: mlua::Value| {
            let globals = lua.globals();
            let print = globals.get::<mlua::Function>("print")?;
            print.call::<()>(Self::encode(&value, Some(true))?)?;
            Ok(())
        });
    }
}
