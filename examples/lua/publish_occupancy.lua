-- Example of publishing occupancy messages at regular intervals

-- These packages are built-in to the Tiny SSE server
local json = require "json"
local http = require "http"
local agent = http.agent {
    headers = {
        ["content-type"] = "application/json"
    }
}

-- The number of active subscribers
local subs = 0

local function pubmsg(msg)
    agent:post("http://127.0.0.1:1983/sse", {
        body = json(msg)
    })
end

function tick(count)
    -- By default the script tick runs every 500ms.
    -- Publish occupancy messages every 10 seconds.
    if count % 20 == 0 then
        pubmsg {
            event = "occupancy",
            data = tostring(subs)
        }
    end
end

function catchup(sub, last_event_id)
    -- Send the current occupancy to the new subscriber
    return {
        {
            event = "occupancy",
            data = tostring(subs)
        }
    }
end

function subscribe(sub)
    subs = subs + 1
    return sub
end

function unsubscribe(sub)
    subs = subs - 1
end
