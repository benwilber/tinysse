# Tiny SSE

A programmable server for Server-Sent Events (SSE).


## Features


- **Customizable Event Routing and Transformation** – Filter, modify, or redirect messages dynamically.
- **Stateful Connection Tracking** – Keep track of active subscribers and their connection state.
- **Graceful Connection Recovery** – Built-in reconnection strategy using `retry` instructions for better reliability.
- **Pluggable Authentication Hooks** – Enforce authentication rules for subscriptions and publishing.
- **Event History & Replay** – Store and replay recent messages for late joiners.
- **Access Control** – Dynamically allow or deny access to certain events based on request properties.


## Examples

### Basic Pub/Sub server
Start the server

```sh
$ tinysse
```

```
2025-02-23T20:27:52.291799Z  INFO tinysse: Listening on 127.0.0.1:1983

```

Start a subscriber

```sh
curl http://127.0.0.1:1983/sse
```
```
: ok
...
```

Publish a message

```sh
curl -X POST -d data="Hello, World" http://127.0.0.1:1983/sse
```
```
{"queued":1,"subscribers":1}
```

Observe the message received by the subscriber

```
: ok

data: Hello, World
...
```

## A Whirlwind Tour

Make a Lua script `script.lua`

Run the server with the script

```sh
tinysse --script script.lua
```

```lua
-- The `uuid` package is built-in to the Tiny SSE server
local uuid = require "uuid"

-- A new message is published
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


## Usage

```sh
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
          The text content of the keep-alive messages sent to clients.
          This text helps clients recognize keep-alive messages and avoid treating them as real events
          
          [env: TINYSSE_KEEP_ALIVE_TEXT=]
          [default: keep-alive]

  -t, --timeout <TIMEOUT>
          The timeout duration for subscriber connections (e.g., 5m, 300s, 10m).
          Connections subscribed for longer than this duration will be closed
          
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
          Optional data to pass to the Lua script as the `cli.script_data` value in the `startup(cli)` function
          
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

  -M, --max-body-size <BYTES>
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

  -D, --serve-root-dir <DIR_PATH>
          Serve static files from the specified directory under the root (/) URL path
          
          [env: TINYSSE_SERVE_ROOT_DIR=]

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
          Set Access-Control-Max-Age header to the specified duration (e.g., 1h, 60s)
          
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

By contributing to Tiny SSE, you agree that your contributions will be licensed under the **BSD-3-Clause License**.

Thank you for helping improve Tiny SSE! 🚀
