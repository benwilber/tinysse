# Built-in Lua packages

The Tiny SSE server comes with several built-in packages.

## Table of Contents

- [`uuid` - Generate UUIDs](#uuid)
- [`json` - Encode and decode JSON](#json)
  - [JSON utility functions](#json-utility-functions)
- [`base64` - Encode and decode base64](#base64)
- [`url` - Parse and construct URLs](#url)
- [`log` - Log messages to the server logger](#log)
- [`http` - Make HTTP requests](#http)
- [`sqlite` (EXPERIMENTAL) - Use an embedded database](#sqlite-experimental)
- [`sleep` Pause script execution](#sleep)
- [`mutex` Lock concurrent operations](#mutex)
- [`fernet` Easy, safe symmetric encryption](#fernet)
- [`template` Use Jinja2 templates](#template)

## `uuid`

Generate UUIDv4 and UUIDv7 strings

```lua
local uuid = require "uuid"

local v4 = uuid.v4() -- v4 UUID
local v7 = uuid.v7() -- v7 UUID

-- Calling the package directly is an alias to `uuid.v4()`
local v4 = uuid()
-- "c53acc2e-c6f8-4812-9ca8-9b41e00a7e19"

```

## `json`

Encode and decode JSON

```lua
local json = require "json"

json.encode {foo = "bar"}
-- '{"foo": "bar"}'

json.decode '{"foo": "bar"}'
-- {foo = "bar"}

json.encode {"foo", "bar"}
-- '["foo", "bar"]'

-- Encode an empty array
json.encode(json.array())
-- '[]'

-- Encode JSON `null`
json.encode {foo = json.null}
-- '{"foo": null}'

-- Calling the package directly is an alias for `json.encode`
json {
	foo = "bar",
	nest = {baz = "bil"},
	null = json.null,
	nums = {1, 2, 3.22},
	bools = {true, false},
	empty = json.array()
}
-- {
--   "foo": "bar",
--   "nest": {
--     "baz": "bil"
--   },
--   "null": null,
--   "nums": [
--     1,
--     2,
--     3.22
--   ],
--   "bools": [
--     true,
--     false
--   ],
--   "empty": []
-- }
```

### JSON utility functions

The `json` package includes utilities for printing Lua values

```lua
local tbl = {foo = "bar", nums = {1, 2, 3}}

-- Print a JSON-encoded repr of this Lua table to stdout
json.print(tbl)
-- stdout: {"foo":"bar","nums":[1,2,3]}

-- Print a pretty-formatted repr of this Lua table to stdout
json.pprint(tbl)
-- stdout:
-- {
--   "foo": "bar",
--   "nums": [
--     1,
--     2,
--     3
--   ]
-- }
```

## `base64`

Encode and decode base64

```lua
local base64 = require "base64"
local s = "some binary string"
local e = base64.encode(s)
local d = base64.decode(e)
assert(s == d)
```

`base64(val)` is an alias for `base64.encode(val)`

The package also supports URL-safe base64 alphabets like:

```lua
local base64 = require("base64").urlsafe()

-- rest of the API is the same
```

## `url`

Parse and construct URLs

```lua
local url = require "url"

-- Encode a URL from parts
url.encode {
  scheme = "https",
  username = "user",
  password = "pass",
  host = "example.com",
  port = 443,
  query = "key=value",
  fragment = "section",
  args = {
    key1 = {
        "value1",
        "value2"
    },
    key2 = {
        "value3"
    }
  }
}
-- "https://user:pass@example.com:443/?key=value&key1=value1&key1=value2&key2=value3#section"
--
-- Note that `query` and `args` are merged if both are provided.

-- Decode a URL into parts
url.decode "https://user:pass@example.com:443/path?key=value#section"
-- {
--   scheme = "https",
--   username = "user",
--   password = "pass",
--   host = "example.com",
--   query = "key=value",
--   fragment = "section",
--   args = {
--     key = {
--       "value"
--     }
--   }
-- }

-- Quote (serialize) a Lua table into query parameters (`application/x-www-form-urlencoded`)
url.quote {
  key = {
    "value1",
    "value2"
  },
  other = {
    "value3"
  }
}
-- "key=value1&key=value2&other=value3"

-- Unquote (deserialize) query parameters into a Lua table
url.unquote "key=value1&key=value2&other=value3"
-- {
--   key = {
--     "value1",
--     "value2"
--   },
--   other = {
--     "value3"
--   }
-- }
```

## `log`

Log messages to the server logger

```lua
local log = require "log"
 
-- Logging with a specific level
log.log(log.INFO, "Custom info log.")

-- Or using the shortcut functions
log.error("An error message.")
log.warn("A warning message.")
log.info("An informational message.")
log.debug("A debug message.")
log.trace("A trace message.")
```

## `http`

Make HTTP requests

**NOTES:**

* The HTTP package does not support streaming request or response bodies.  The entire body is always buffered into server memory.

 ```lua
 local http = require "http"
 local json = require "json"

 -- Perform an HTTP GET request
 local r = http.request(
     "GET",
     "http://httpbin.org/get",
     {
         args = {key = "value"} -- Appends ?key=value to the URL
         headers = {accept = "application/json"}
     }
 )

 print("Status:", r.status)
 print("Headers:", json(r.headers))
 print("Body:", r.body)

 -- Perform an HTTP POST request with a body
 local r = http.request(
     "POST",
     "https://httpbin.org/post",
     {
         headers = {["content-type"] = "application/json"},
         body = json {key = "value"}
     }
 )
 print("Status:", r.status)
 print("Body:", r.body)
 ```

 The `http.request` method is asynchronous and returns a table with the response details.

 The standard HTTP methods are supported: `GET`, `HEAD`, `OPTIONS`, `POST`, `PUT`, `PATCH`, and `DELETE`.
 
```lua
-- Convenience functions
http.get(url, opts)
http.head(url, opts)
http.options(url, opts)
http.post(url, opts)
http.put(url, opts)
http.patch(url, opts)
http.delete(url, opts) 
```
 
Reusable client
 
```lua
-- All requests made with this agent will
-- reuse the internal connection pool and apply
-- the specified options (unless overridden per-request).
local agent = http.agent {
    headers = {
    	["user-agent"] = "my-user-agent",
    	["content-type"] = "application/json"
    }
}
local r = agent:post("https://httpbin.org/post", {
    body = json {key = "value"}
})
```

## `sqlite` (EXPERIMENTAL / NOT threadsafe (yet))

**NOTE:** The SQLite package does not support transactions.

```lua
local uuid = require "uuid"
local sqlite = require "sqlite"
local json = require "json"

local db = sqlite.open(":memory:")
db:exec [[
  create table msg (
    id text primary key
  )
]]

function tick(count)
    local rows = db:query("select count(*) from msg")
    
    if #rows > 0 then
        -- See: BUILTINS.md#json-utility-functions
        json.pprint(rows)
    end
end

function publish(pub)
  pub.msg.id = uuid()
  db:exec("insert into msg (id) values (?)", {pub.msg.id})
  return pub
end
```

**NOTE:** SQL `NULL` values can be represented in Lua via the `sqlite.null` constant.

```lua
db:exec("insert into tbl (name) values (?)", {db.null})
-- Translates to the following SQL:
-- insert into tbl (name) values (NULL)
```

## `sleep`

Pause Script Execution

```lua
local sleep = require "sleep"
```

The `sleep` package provides a single function, `sleep`, that pauses script execution for a specified number of milliseconds. The function is useful for delaying script execution or for simulating slow operations.

```lua
sleep(1000) -- sleep for 1 second
```

**NOTE:** The `sleep` function is asynchronous and does not block the overall server's execution. It will only block the operations within the current script invocation context.

## `mutex`

Lock Concurrent Operations

```lua
local mutex = require "mutex"
```


The `mutex` package provides a single function, `mutex()`, that will create a new mutex-based lock. The returned function can be called with a function argument to synchronize concurrent operations on a shared resource.

```lua
local mutex = require "mutex"
local lock = mutex()

local res = lock(function()
  -- Critical section
  -- This code will be concurrency-safe
  return do_thing_with_resource() -- Result will be returned to the outer scope `res`
end)
```

**NOTE:** Beware of recursive locking (calling `lock()` again inside the `lock`'s critical section). The code will deadlock.

## `fernet`

Easy, safe symmetric encryption

Fernet provides symmetric-authenticated-encryption with an API that makes misusing it difficult. It is based on a public specification and there are interoperable implementations in Rust, Python, Ruby, JavaScript, Go, and many others.

```lua
local fernet = require "fernet"

local key = fernet.genkey() -- Store this somewhere safe
local f = fernet(key)
local secret = "my secret message"
local encrypted = f:encrypt(secret)
local decrypted = f:decrypt(encrypted)
assert(secret == decrypted)
```


## `template`

Use Jinja2 templates

Template provides a simple interface to the Jinja2 templating engine.

```lua
local template = require "template"
```

```lua
-- The template package itself provides a single top-level function `renderstring(str, ctx)`
local rendered = template.renderstring("Hello, {{ name }}!", { name = "World" })
assert(rendered == "Hello, World!")
```

Using a Library for engine customization

```lua
local template = require("template").library {
  -- Load templates from a filesystem directory
  directory = "path/to/templates",

  -- Or define templates inline
  templates = {
    base = [[
      This is the base template
      {% block user %}{% endblock %}
    ]],

    user = [[
      {% extends "base" %}
      {% block user %}
          Hello, {{ user.name }}!
      {% endblock %}
    ]]
  },

  -- Set the template autoescaping behavior.
  -- Can be one of "html", "json", or "none".  Defaults to "html"
  autoescape = "html",

  keep_trailing_newline = true,
  trim_blocks = true,
  lstrip_blocks = true
}

local rendered = template:render("user", { user = { name = "Friend" } })
assert(rendered == [[
This is the base template
Hello, Friend!
]])

-- Can also render an inline template with the library
local rendered = template:renderstring([[
  {% extends "base" %}
  {% block user %}
    Good Job, {{ user.name }}!
  {% endblock %}
]], { user = { name = "Friend" } })
assert(rendered == [[
This is the base template
Good Job, Friend!
]])

-- Add and remove templates to/from the library dynamically
template:add("user", [[
  {% extends "base" %}
  {% block user %}
    Good Job, {{ user.name }}!
  {% endblock %}
]])
template:remove("user")

-- Remove all templates from the library
template:clear()
```
