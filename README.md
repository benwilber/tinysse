# tinysse

## Usage

```shell
$ tinysse --help
Tiny SSE Server

This server supports Lua scripting for customization. Use the following options to configure the server.

Duration Format: Time-related options (e.g., keep-alive, timeout) use a human-readable format: - `1s` means 1 second. - `1000ms` means 1000 milliseconds (can be shortened to `1s`). - Other examples: `5m` (5 minutes), `2h` (2 hours), `3d` (3 days).

Use these formats consistently for options like `--keep-alive`, `--timeout`, etc.

Usage: tinysse [OPTIONS]

Options:
  -l, --listen <ADDR:PORT>
          The address and port for the HTTP server to listen on
          
          [env: TINYSSE_LISTEN=]
          [default: 127.0.0.1:1983]

  -L, --log-level <LEVEL>
          The logging level for the server. Possible values: ERROR, WARN, INFO, DEBUG, TRACE
          
          [env: TINYSSE_LOG_LEVEL=]
          [default: INFO]

  -k, --keep-alive <INTERVAL>
          The interval between keep-alive messages sent to clients (e.g., 60s, 2m).
          Keep-alive messages are sent periodically to ensure that clients remain connected.
          
          [env: TINYSSE_KEEP_ALIVE=]
          [default: 60s]

  -K, --keep-alive-text <TEXT>
          The text content of the keep-alive messages sent to clients.
          This text helps clients recognize keep-alive messages and avoid treating them as real events.
          
          [env: TINYSSE_KEEP_ALIVE_TEXT=]
          [default: keep-alive]

  -t, --timeout <TIMEOUT>
          The timeout duration for idle connections (e.g., 5m, 300s, 10m).
          Connections that remain idle longer than this duration will be closed.
          
          [env: TINYSSE_TIMEOUT=]
          [default: 5m]

  -r, --timeout-retry <RETRY>
          The retry interval sent to clients after a connection timeout (e.g., 0s, 2s).
          This interval instructs clients how long to wait before attempting to reconnect.
          
          [env: TINYSSE_TIMEOUT_RETRY=]
          [default: 0s]

  -c, --capacity <CAPACITY>
          The capacity of the server's internal event queue
          
          [env: TINYSSE_CAPACITY=]
          [default: 32]

  -s, --script <PATH>
          The path to a Lua script for server customization
          
          [env: TINYSSE_SCRIPT=]

      --unsafe-script
          Allow the Lua script to load (require) native code, such as shared (.so) libraries. Enabling this can pose security risks, as native code can execute arbitrary operations. Use this option only if you trust the Lua script and need it to load native modules.
          
          [env: TINYSSE_UNSAFE_SCRIPT=]

  -h, --help
          Print help (see a summary with '-h')
```

## Example

A Lua script to handle messages

```lua
-- script.lua
local log = require "log"
local json = require "json"
local uuid = require "uuid"

function publish(pub)
    -- For example, add unique IDs to the publisher and message
    pub.id = uuid()
    pub.msg.id = uuid()

    log.info(("publish: pub=%s"):format(json(pub)))

    -- Return the publish request
    return pub
end

function subscribe(sub)
    -- Add a unique ID to the subscriber
    sub.id = uuid()

    log.info(("subscribe: sub=%s"):format(json(sub)))

    -- Return the subscribe request
    return sub
end

function message(pub, sub)
    -- `pub` and `sub` are the request tables returned
    -- from the publish and subscribe methods.
    log.info(("message: pub=%s, sub=%s"):format(json(pub), json(sub)))

    -- Return the publish request
    return pub
end
```

Start the server
 
```shell
$ tinysse --script script.lua
2024-12-06T18:00:44.756171Z  INFO tinysse: Listening on 127.0.0.1:1983
```

Start a subscriber

```shell
$ curl -is http://127.0.0.1:1983/subscribe
HTTP/1.1 200 OK
content-type: text/event-stream
cache-control: no-cache
transfer-encoding: chunked
date: Fri, 06 Dec 2024 18:03:06 GMT

: ok
```

Publish a message

```shell
$ curl -X POST -d data=Hello http://127.0.0.1:1983/publish
{"queued": 1, "subscribers": 1}
```

Observe the message in the subscriber

```shell
$ curl -is http://127.0.0.1:1983/subscribe
HTTP/1.1 200 OK
content-type: text/event-stream
cache-control: no-cache
transfer-encoding: chunked
date: Fri, 06 Dec 2024 18:03:06 GMT

: ok

id: bc349484-2cfe-4277-8714-ef72818b238d
data: Hello
```