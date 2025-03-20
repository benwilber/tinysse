-- Example of publishing messages from the Lua program

-- These packages are built-in to the Tiny SSE server
local uuid = require "uuid"
local json = require "json"
local http = require "http"
local agent = http.agent {
    headers = {
        ["content-type"] = "application/json"
    }
}

local function pubmsg(msg)
    agent:post("http://127.0.0.1:1983/sse", {
        body = json(msg)
    })
end

function subscribe(sub)
    -- Give the subscriber a unique ID
    sub.id = uuid()

    -- Publish a message when a new subscriber connects
    pubmsg {
        event = "subscribe",
        data = json {
            id = sub.id
        }
    }

    return sub
end

function unsubscribe(sub)
    -- Publish a message when a subscriber disconnects
    pubmsg {
        event = "unsubscribe",
        data = json {
            id = sub.id
        }
    }
end
