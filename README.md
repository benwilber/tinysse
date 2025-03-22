# Tiny SSE

A programmable server for Server-Sent Events (SSE).

## Features


- **Flexible Message Handling** – Filter, modify, redirect, and replay messages dynamically.  
- **Reliable Connections** – Track subscribers, support reconnections, and maintain session state.  
- **Secure Access Control** – Enforce authentication, authorization, and event-based restrictions.  
- **Customizable Behavior** – Use hooks to modify messages and manage subscriptions programmatically.  


## Table of Contents

- [Features](#features)
- [Installation](#installation)
  - [Homebrew](#homebrew-macos)
  - [Building](#building)
- [Examples](#examples)
  - [Basic Pub/Sub server](#basic-pubsub-server)
  - [A Whirlwind Tour](#a-whirlwind-tour)
- [HTTP API](#http-api)
  - [Publishing messages](#publishing-messages)
    - [SSE message fields](#sse-message-fields)
  - [Subscribing to messages](#subscribing-to-messages)
- [Lua API](#lua-api)
  - [`startup(cli)`](#startupcli)
  - [`tick(count)`](#tickcount)
  - [`publish(pub)`](#publishpub)
  - [`subscribe(sub)`](#subscribesub)
  - [`catchup(sub, last_event_id)`](#catchupsub-last_event_id)
  - [`message(pub, sub)`](#messagepub-sub)
  - [`unsubscribe(sub)`](#unsubscribesub)
  - [`timeout(sub, elapsed)`](#timeoutsub-elapsed)
- [Lua API Built-ins](BUILTINS.md)
- [Usage](#usage)
- [Contributing to Tiny SSE](#contributing-to-tiny-sse)
  - [Getting Started](#getting-started)
  - [Reporting Issues](#reporting-issues)
  - [Feature Requests](#feature-requests)
  - [Code Contributions](#code-contributions)

## Installation

[Archives of binary releases](https://github.com/benwilber/tinysse/releases) are available for Linux, macOS, and Windows.

### Homebrew (macOS)

```sh
brew tap benwilber/tinysse
brew install benwilber/tinysse/tinysse-bin
tinysse --help
```

### Building

The project can be built with the standard Rust/Cargo toolchain:

```sh
git clone https://github.com/benwilber/tinysse.git
cd tinysse
cargo build --release
./target/release/tinysse --help
```

## Examples

### Basic Pub/Sub server
Start the server

```sh
$ tinysse
```

```
INFO tinysse: Listening on 127.0.0.1:1983
```

1) Start a subscriber

```sh
curl http://127.0.0.1:1983/sse
```
```
: ok
...
```

2) Publish a message

```sh
curl -X POST -d data="Hello, World" http://127.0.0.1:1983/sse
```
```
{"queued":1,"subscribers":1}
```

3) Observe the message received by the subscriber

```
: ok

data: Hello, World
...
```

## A Whirlwind Tour

Make a Lua script `script.lua`

Run the server with the script

```sh
$ tinysse --script script.lua
```

```lua
-- The `uuid` package is built-in to the Tiny SSE server
local uuid = require "uuid"

-- A message is published
function publish(pub)
  -- Set a unique ID on the publish request.
  -- This can later be referenced in the `message(pub, sub)`
  -- function to correlate the publish request with message
  -- delivery to subscribers
  pub.id = uuid()

  -- We can override the data in the SSE message
  pub.msg.data = "Hello, Universe!"

  -- If the publisher did not set a message ID, then we can set one here.
  -- This will be the `id: <id>` line in the SSE message.
  if not pub.msg.id then
    pub.msg.id = uuid()
  end

  -- We can set a custom event
  pub.msg.event = "custom-event"

  -- Comments too
  pub.msg.comment = {"This is a comment", "Another comment!"}

  -- Return the pub request to the server or it
  -- will be rejected and not delivered to any subscribers
  return pub
end

-- A new subscriber connects
function subscribe(sub)
  -- Set a unique ID on the subscriber.
  sub.id = uuid()

  -- Return the sub request to the server or it
  -- will be rejected and the client will be disconnected immediately
  return sub
end

-- A message is delivered to a subscriber
function message(pub, sub)
  print("Publish ID:", pub.id)
  print("Message ID:", pub.msg.id)
  print("Subscriber ID:", sub.id)

  -- Return the pub request to the server or
  -- the subscriber will not receive the message
  -- (but will still remain connected for subsequent messages)
  return pub
end

-- A subscriber disconnects
function unsubscribe(sub)
  print("Unsubscribed:", sub.id)
end
```

## HTTP API
### Publishing messages
The server supports publishing SSE messages via HTTP `POST` to the URL path configured by the `--pub-path=<path>` option (defaults to `/sse`).

It accepts data encoded as both `application/x-www-form-urlencoded` and `application/json`.  The specific content type must always be indicated in the request or it will be rejected.

```curl
curl -i -X POST \
  --header "content-type: application/json" \
  --data-raw '{"data": "Hello World"}' \
  http://127.0.0.1:1983/sse
```

A successful publish will respond with a `202 Accepted` status code and an `application/json` body with the current number of subscribers and the number of messages in the queue that have not been delivered to all subscribers (yet).

```
HTTP/1.1 202 Accepted
content-type: application/json
content-length: 31

{"queued": 1, "subscribers": 1}
```

The size of the internal message queue can be configured with the `--capacity=<size>` option.  It defaults to 256.

#### SSE message fields

All fields are optional but at least one must be provided or the message will be rejected with a `400 Bad Request` error.


```json
{
  "id": "some-id",
  "event": "custom-event",
  "data": "Some data",
  "comment": ["First comment", "Second comment"]
}
```

Equivalent message as `application/x-www-form-urlencoded`:

```form
id=some-message-id
&event=custom-event
&data=Some%20data
&comment=First%20comment
&comment=Second%20comment
```

`data` containing newlines is automatically split across multiple `data:` lines in the SSE message.

### Subscribing to messages

The server supports subscribing to SSE messages via HTTP `GET` to the URL path configured by the `--sub-path=<path>` option (defaults to `/sse`).

```curl
curl -i http://127.0.0.1:1983/sse

HTTP/1.1 200 OK
content-type: text/event-stream
cache-control: no-cache

: ok

id: some-id
event: custom-event
data: Some data
: First comment
: Second comment

: keep-alive
```

Upon successful subscription, the server will immediately respond with the SSE comment `ok` indicating that the connection is established and waiting for new messages.

Keep-alive messages (SSE comments) are sent periodically to ensure the connection stays open and is not closed by intermediate proxies due to socket inactivity.  These messages are configurable with the `--keep-alive` and `--keep-alive-text` options.

## Lua API

The server can function as just a simple SSE pub/sub server without using the Lua API.  However, much of the advanced functionality (authorization, message routing, etc.) requires writing Lua code to implement custom behaviors.  The server is asynchronous and invokes global Lua functions defined in the script given by the `--script=<path>` option when various events occur.  The server will provide arguments to the functions with context of the event.

The program runs in a single Lua context for the lifetime of the server so that a global state is shared across the various function calls.

### `startup(cli)`

This is the first function called by the server immediately after it
begins listening on the configured address and port (default: `127.0.0.1:1983`) and
before the socket accepts any client connections.  It will be called only
once during the server lifetime, and will provide the CLI options to the program
as a Lua table `cli`.
The server will not accept a return value from this function.  However, it
will abort if the function raises a Lua error.

```lua
function startup(cli)
  -- The `cli` table looks like:
  {
    keep_alive_text = "keep-alive",
    script = "script.lua",
    script_tick = 500,
    log_level = "INFO",
    pub_path = "/sse",
    sub_path = "/sse",
    keep_alive = 60000,
    timeout_retry = 0,
    timeout = 300000,
    serve_static_path = "/",
    capacity = 256,
    listen = "127.0.0.1:1983",
    unsafe_script = false
  }
end
```

### `tick(count)`

A periodic event that allows the Lua script to "wake up" and perform background tasks at regular intervals (default `500ms`).  It provides a single argument `count` which is the number of times the tick function has been invoked (including the current) since the server started.

```lua
function tick(count)
  -- Do background work here
end
```

### `publish(pub)`

Called when a client wants to publish a message.  It provides a single argument `pub` which is a Lua table containing context of the publish request.  The function is free to modify the request however it needs, but it must return it (modified or not) to the server or the publish request will be rejected with a `403 Forbidden` error and the message will not be delivered to any subscribers.

**NOTE**: Changes to the inner `req` table will not be preserved.

```lua
function publish(pub)
  -- The `pub` table looks like
  {
    req = {
      headers = {
        ["content-type"] = "application/json",
        ["content-length"] = "24",
        accept = "*/*",
        host = "127.0.0.1:1983",
        ["user-agent"] = "curl/8.7.1"
      },
      query = "",
      path = "/sse",
      addr = {
        ip = "127.0.0.1",
        port = 59615
      },
      method = "POST"
    },
    msg = {
      data = "Hello, World"
    }
  }
  
  -- The function is free to modify this table however it needs, but it
  -- must return it to the server or the message will be rejected.
  return pub
end
```

### `subscribe(sub)`

Called when a new subscriber connects.  It provides a single argument `sub` which is a Lua table containing context of the subscribe request.  The function is free to modify the request however it needs, but it must return it (modified or not) to the server or the connection will be rejected with a `403 Forbidden` error and the client will be disconnected immediately.

**NOTE**: Changes to the inner `req` table will not be preserved.

```lua
function subscribe(sub)
  -- The `sub` table looks like:
  {
    req = {
      query = "",
      headers = {
        ["user-agent"] = "curl/8.7.1",
        accept = "*/*",
        host = "127.0.0.1:1983"
      },
      path = "/sse",
      addr = {
        ip = "127.0.0.1",
        port = 59632
      },
      method = "GET"
    }
  }
  
  -- The function is free to modify this table however it needs, but it
  -- must return it to the server or the subscribe request will be rejected.
  return sub
end
```

### `catchup(sub, last_event_id)`

Called immediately after subscribe when the client includes a Last-Event-ID either as a request header `Last-Event-ID:` or a query parameter `?last_event_id=`.  When both are given the header will take precedence.  This function is expected to return an array table of SSE messages to "catch-up" the subscriber with any messages they might have missed due to reconnection, or just recent history.

**NOTE:** The `message(pub, sub)` function will not be called for messages delivered from the `catchup(sub, last_event_id)` function.

```lua
function catchup(sub, last_event_id)
  local msgs = {}

  -- For instance, "catch-up" subscriber with the 10 most recent messages
  for i=1,10 do
    table.insert(msgs, {
      id = "some-id-" .. i,
      event = "some-event",
      data = "some data"
    })
  end

  return msgs
end
```

### `message(pub, sub)`

Called before delivering a message to a subscriber. Receives `pub` and `sub`, the tables returned from the `publish` and `subscribe` functions. Modifications to these tables affect only this subscriber, not others receiving the same message. Typically used for routing and subscriber-specific adjustments.

**NOTE**: Changes to the inner `req` tables will not be preserved.

```lua
function message(pub, sub)
  -- Subscriber-specific logic such as routing, message modifications, etc.
  
  -- Returning `nil` (or just nothing at all) will prevent this subscriber from receiving
  -- the message.  Returning the `pub` table to the server will continue with
  -- delivery of the (possibly modified) SSE message to the subscriber.
  return pub
end
```

### `unsubscribe(sub)`

Called when a subscriber disconnects.  The server provides a single argument `sub` which is the Lua table returned from the `subscribe` function.  It does not accept any return value.

```lua
function unsubscribe(sub)
  -- Client has unsubscribed (disconnected) from the SSE server.
end
```

### `timeout(sub, elapsed)`

Called when a subscriber disconnects as result of an SSE timeout.  The server provides two arguments, `sub` and `elapsed`.  `sub` is the table returned from the `subscribe` function, and `elapsed` is the total milliseconds that the subscriber was connected.  The server accepts an optional return value which is the number of milliseconds that the client should wait before reconnecting.  If not given, it will default to the value given by the `--timeout-retry` option.

**NOTE:** The `unsubscribe(sub)` function will be called immediately after this.

```lua
function timeout(sub, elapsed)
  -- Subscriber timed-out and was disconnected.
end
```

For advanced usage, see the [Lua API Built-ins](BUILTINS.md#built-in-lua-packages) and the [Lua examples](examples/lua)


## Usage

```text
$ tinysse --help
Tiny SSE

A programmable server for Server-Sent Events (SSE).

Usage: tinysse [OPTIONS]

Options:
  -l, --listen <ADDR:PORT>
          The address and port for the HTTP server to listen
          
          [env: TINYSSE_LISTEN=]
          [default: 127.0.0.1:1983]

  -L, --log-level <LEVEL>
          The logging level for the server. Possible values: ERROR, WARN, INFO, DEBUG, TRACE
          
          [env: TINYSSE_LOG_LEVEL=]
          [default: INFO]

  -k, --keep-alive <INTERVAL>
          The interval between keep-alive messages sent to clients (e.g., 60s, 2m).
          Keep-alive messages are sent periodically to ensure that clients remain connected
          
          [env: TINYSSE_KEEP_ALIVE=]
          [default: 60s]

  -K, --keep-alive-text <TEXT>
          The text of the keep-alive comment sent to clients.
          
          [env: TINYSSE_KEEP_ALIVE_TEXT=]
          [default: keep-alive]

  -t, --timeout <TIMEOUT>
          The timeout duration for subscriber connections (e.g., 5m, 300s, 10m).
          Connections open for longer than this duration will be closed
          
          [env: TINYSSE_TIMEOUT=]
          [default: 5m]

  -r, --timeout-retry <RETRY>
          The retry delay sent to clients after a connection timeout (e.g., 0s, 2s).
          This delay instructs clients how long to wait before attempting to reconnect.
          Setting this to 0s instructs the client to reconnect immediately
          
          [env: TINYSSE_TIMEOUT_RETRY=]
          [default: 0s]

  -c, --capacity <CAPACITY>
          The capacity of the server's internal message queue
          
          [env: TINYSSE_CAPACITY=]
          [default: 256]

  -s, --script <FILE_PATH>
          The path to a Lua script for server customization
          
          [env: TINYSSE_SCRIPT=]

      --script-data <DATA>
          Optional data to pass to the Lua script as the `opts.script_data` value in the `startup(opts)` function
          
          [env: TINYSSE_SCRIPT_DATA=]

      --script-tick <INTERVAL>
          The interval between Lua script ticks (e.g., 1s, 500ms). The script tick is a periodic event that allows the Lua script to perform
          background tasks in the `tick(count)` function
          
          [env: TINYSSE_SCRIPT_TICK=]
          [default: 500ms]

      --unsafe-script
          Allow the Lua script to load (require) native code, such as shared (.so) libraries. Enabling this can pose security risks, as
          native code can execute arbitrary operations. Use this option only if you trust the Lua script and need it to load native modules
          
          [env: TINYSSE_UNSAFE_SCRIPT=]

  -m, --max-body-size <BYTES>
          The maximum size of the publish request body that the server will accept (e.g., 32KB, 1MB)
          
          [env: TINYSSE_MAX_BODY_SIZE=]
          [default: 64KB]

  -P, --pub-path <URL_PATH>
          The URL path for publishing messages via POST
          
          [env: TINYSSE_PUB_PATH=]
          [default: /sse]

  -S, --sub-path <URL_PATH>
          The URL path for subscribing to messages via GET
          
          [env: TINYSSE_SUB_PATH=]
          [default: /sse]

  -D, --serve-static-dir <DIR_PATH>
          Serve static files from the specified directory under the path specified by `--serve-static-path`
          
          [env: TINYSSE_SERVE_STATIC_DIR=]

  -U, --serve-static-path <URL_PATH>
          The URL path under which to serve static files from the directory specified by `--serve-static-dir`
          
          [env: TINYSSE_SERVE_STATIC_PATH=]
          [default: /]

      --cors-allow-origin <ORIGINS>
          Set Access-Control-Allow-Origin header to the specified origin(s)
          
          [env: TINYSSE_CORS_ALLOW_ORIGIN=]
          [default: *]

      --cors-allow-methods <METHODS>
          Set Access-Control-Allow-Methods header to the specified method(s)
          
          [env: TINYSSE_CORS_ALLOW_METHODS=]
          [default: "GET, HEAD, POST"]

      --cors-allow-headers <HEADERS>
          Set Access-Control-Allow-Headers header to the specified header(s). (e.g., Cookie,Authorization)
          
          [env: TINYSSE_CORS_ALLOW_HEADERS=]
          [default: *]

      --cors-allow-credentials
          Set Access-Control-Allow-Credentials header to true. Cannot be set if Access-Control-Allow-Origin or Access-Control-Allow-Headers
          is set to '*' (any)
          
          [env: TINYSSE_CORS_ALLOW_CREDENTIALS=]

      --cors-max-age <DURATION>
          Set Access-Control-Max-Age header to the specified duration (e.g., 1h, 60s). Set to 0s to disable browsers from caching preflight OPTIONS requests
          
          [env: TINYSSE_CORS_MAX_AGE=]
          [default: 0s]

  -h, --help
          Print help (see a summary with '-h')
```

## Contributing to Tiny SSE

Thank you for your interest in contributing to Tiny SSE! We welcome all contributions, including bug reports, feature requests, documentation improvements, and code contributions.

### Getting Started

1. Fork the repository and create a new branch for your changes.
2. Make your modifications and ensure they follow Rust (and Lua) best practices.
3. Run tests to verify your changes with `cargo test`.
4. Format your code using `cargo fmt` and check for issues with `cargo check`.
5. Submit a pull request with a clear description of your changes.

### Reporting Issues

If you encounter a bug, please open an issue and include:

- A clear description of the problem.
- Steps to reproduce the issue.
- Expected vs. actual behavior.
- Any relevant logs or error messages.

### Feature Requests

We welcome feature suggestions! Before submitting a request, check if an issue already exists. Provide a detailed explanation of how the feature benefits the project.

### Code Contributions

- Follow Rust (and Lua) best practices and maintain code clarity.
- Use descriptive commit messages summarizing your changes.
- Write tests for new features or bug fixes.
- Keep discussions respectful and relevant.

By contributing to Tiny SSE, you agree that your contributions will be licensed under the **[Apache-2.0 license](LICENSE)**.

Thank you for helping improve Tiny SSE!