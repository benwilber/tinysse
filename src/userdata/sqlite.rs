#![allow(unused_doc_comments)]
use mlua::LuaSerdeExt as _;
use tokio_sqlite as sqlite;

/// The SQLite database interface.
pub struct Sqlite;

impl Sqlite {
    /// Opens a SQLite database at the given file path.
    ///
    /// # Arguments
    ///
    /// * `path` - A reference to a path specifying the SQLite database file.
    ///            The special value `:memory:` can be used to open an in-memory database.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Connection` object on success or a `sqlite::Error` on failure.
    pub async fn open<P>(path: P) -> Result<Connection, sqlite::Error>
    where
        P: AsRef<std::path::Path>,
    {
        Connection::open(path).await
    }
}

impl mlua::UserData for Sqlite {
    /// Adds Lua methods for the `Sqlite` struct.
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        /// Opens a SQLite database from Lua.
        methods.add_async_function("open", |_lua, path: String| async move {
            Self::open(&path).await.map_err(mlua::Error::external)
        });
    }
}

/// A struct representing a connection to a SQLite database.
pub struct Connection {
    inner: sqlite::Connection,
}

impl Connection {
    /// Opens a SQLite database connection.
    ///
    /// # Arguments
    ///
    /// * `path` - A reference to a path specifying the SQLite database file.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Connection` object on success or a `sqlite::Error` on failure.
    pub async fn open<P>(path: P) -> Result<Self, sqlite::Error>
    where
        P: AsRef<std::path::Path>,
    {
        sqlite::Connection::open(path)
            .await
            .map(|conn| Connection { inner: conn })
    }

    /// Executes a SQL statement with optional parameters.
    ///
    /// # Arguments
    ///
    /// * `stmt` - A SQL statement to execute.
    /// * `args` - Parameters for the SQL statement.
    ///
    /// # Returns
    ///
    /// A `Result` containing the status of the execution or a `sqlite::Error`.
    pub async fn exec<S, A>(&mut self, stmt: S, args: A) -> Result<sqlite::Status, sqlite::Error>
    where
        S: Into<String>,
        A: Into<Vec<sqlite::Value>>,
    {
        self.inner.execute(stmt, args).await
    }

    /// Executes a query and returns the result rows.
    ///
    /// # Arguments
    ///
    /// * `stmt` - A SQL query statement.
    /// * `args` - Parameters for the SQL query.
    ///
    /// # Returns
    ///
    /// A `Result` containing the rows or a `sqlite::Error`.
    pub async fn query<S, A>(&mut self, stmt: S, args: A) -> Result<sqlite::Rows, sqlite::Error>
    where
        S: Into<String>,
        A: Into<Vec<sqlite::Value>>,
    {
        self.inner.query(stmt, args).await
    }
}

impl mlua::UserData for Connection {
    /// Adds Lua methods for the `Connection` struct.
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        /// Executes a SQL statement with parameters from Lua.
        methods.add_async_method_mut(
            "exec",
            |lua, mut this, (stmt, args): (String, Option<mlua::Table>)| async move {
                let args = to_sqlite_args(&lua, &args)?;

                match this.exec(stmt, args).await {
                    Ok(status) => to_lua_status(&lua, &status),
                    Err(e) => Err(mlua::Error::external(e)),
                }
            },
        );

        /// Executes a query and returns the result rows to Lua.
        methods.add_async_method_mut(
            "query",
            |lua, mut this, (stmt, args): (String, Option<mlua::Table>)| async move {
                let args = to_sqlite_args(&lua, &args)?;

                match this.query(stmt, args).await {
                    Ok(rows) => to_lua_rows(&lua, rows).await,
                    Err(e) => Err(mlua::Error::external(e)),
                }
            },
        );
    }
}

/// Converts a row's columns and values to a Lua table.
///
/// # Arguments
///
/// * `lua` - The Lua context.
/// * `columns` - Column names from the database.
/// * `values` - Values corresponding to the columns.
///
/// # Returns
///
/// A `Result` containing a Lua table or an error.
fn to_lua_row(
    lua: &mlua::Lua,
    columns: &[String],
    values: &[sqlite::Value],
) -> Result<mlua::Table, mlua::Error> {
    let table = lua.create_table()?;

    for (column, value) in columns.iter().zip(values) {
        table.set(column.to_string(), sqlite_to_lua(lua, value)?)?;
    }

    Ok(table)
}

/// Converts rows from SQLite into a Lua table.
///
/// # Arguments
///
/// * `lua` - The Lua context.
/// * `rows` - SQLite rows to convert.
///
/// # Returns
///
/// A `Result` containing a Lua table or an error.
async fn to_lua_rows<'a>(
    lua: &mlua::Lua,
    mut rows: sqlite::Rows<'a>,
) -> Result<mlua::Table, mlua::Error> {
    let table = lua.create_table()?;
    table.set_metatable(Some(lua.array_metatable()));

    while let Some(row) = rows.next().await {
        let row = row.map_err(mlua::Error::external)?;
        let row = to_lua_row(lua, rows.columns(), row.values())?;
        table.push(row)?;
    }

    Ok(table)
}

/// Converts SQLite status to a Lua table.
///
/// # Arguments
///
/// * `lua` - The Lua context.
/// * `status` - SQLite status to convert.
///
/// # Returns
///
/// A `Result` containing a Lua table or an error.
fn to_lua_status(lua: &mlua::Lua, status: &sqlite::Status) -> Result<mlua::Table, mlua::Error> {
    let table = lua.create_table()?;
    table.set("rowsaffected", status.rows_affected()).ok();
    table.set("lastinsertid", status.last_insert_id()).ok();
    Ok(table)
}

/// Converts Lua arguments to SQLite values.
///
/// # Arguments
///
/// * `lua` - The Lua context.
/// * `table` - An optional Lua table containing the arguments.
///
/// # Returns
///
/// A `Result` containing a vector of SQLite values or an error.
fn to_sqlite_args(
    lua: &mlua::Lua,
    table: &Option<mlua::Table>,
) -> Result<Vec<sqlite::Value>, mlua::Error> {
    if let Some(table) = table {
        let mut args = Vec::new();

        for value in table.sequence_values::<mlua::Value>() {
            match value {
                Ok(value) => {
                    args.push(lua_to_sqlite(lua, &value)?);
                }
                Err(e) => return Err(mlua::Error::external(e)),
            }
        }

        Ok(args)
    } else {
        Ok(Vec::new())
    }
}

/// Converts a Lua value to a SQLite value.
///
/// # Arguments
///
/// * `lua` - The Lua context.
/// * `value` - A Lua value.
///
/// # Returns
///
/// A `Result` containing a SQLite value or an error.
fn lua_to_sqlite(_lua: &mlua::Lua, value: &mlua::Value) -> Result<sqlite::Value, mlua::Error> {
    Ok(match value {
        mlua::Value::String(v) => sqlite::Value::Text(v.to_string_lossy()),
        mlua::Value::Integer(v) => sqlite::Value::Integer(*v),
        mlua::Value::Number(v) => sqlite::Value::Real(*v),
        mlua::Value::Boolean(v) => sqlite::Value::Integer(if *v { 1 } else { 0 }),
        mlua::Value::Nil => sqlite::Value::Null,
        mlua::Value::LightUserData(v) if *v == mlua::LightUserData(std::ptr::null_mut()) => {
            sqlite::Value::Null
        }
        v => {
            return Err(mlua::Error::external(format!(
                "cannot convert Lua type '{}' to SQLite",
                v.type_name()
            )))
        }
    })
}

/// Converts a SQLite value to a Lua value.
///
/// # Arguments
///
/// * `lua` - The Lua context.
/// * `value` - A SQLite value.
///
/// # Returns
///
/// A `Result` containing a Lua value or an error.
fn sqlite_to_lua(lua: &mlua::Lua, value: &sqlite::Value) -> Result<mlua::Value, mlua::Error> {
    Ok(match value {
        sqlite::Value::Null => mlua::Value::NULL,
        sqlite::Value::Integer(v) => mlua::Value::Integer(*v),
        sqlite::Value::Real(v) => mlua::Value::Number(*v),
        sqlite::Value::Text(v) => mlua::Value::String(lua.create_string(v)?),
        sqlite::Value::Blob(v) => mlua::Value::String(lua.create_string(v)?),
    })
}
