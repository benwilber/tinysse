-- Example of a simple channel-based pub/sub system
-- 
-- Publish a message to a specific channel
-- curl -X POST -d data="Hello, World" http://127.0.0.1:1983/sse?chan=chan-1
-- 
-- Subscribe to the channel
-- curl http://127.0.0.1:1983/sse?chan=chan-1

-- These packages are built-in to the Tiny SSE server
local uuid = require "uuid"
local url = require "url"

-- Table to map subscribers to channels
local subs = {}

-- Helper function to extract the channels from the request query string
local function channels(req)
    -- Returns a table of channels given in the query string like
    -- `?chan=chan-1&chan=chan-2`
    return url.unquote(req.query).chan or {}
end

-- Add a subscriber to a channel
local function addsub(sub, chan)
    -- Initialize the channel if it doesn't exist
    if not subs[chan] then
        subs[chan] = {}
    end

    subs[chan][sub.id] = true
end

-- Check if a subscriber is in a channel
local function issub(sub, chan)
    return subs[chan] and subs[chan][sub.id]
end

-- Remove a subscriber from a channel
local function delsub(sub, chan)
    if subs[chan] then
        subs[chan][sub.id] = nil

        -- Clean up empty channels
        if not next(subs[chan]) then
            subs[chan] = nil
        end
    end
end

function subscribe(sub)
    -- Give the subscriber a unique ID
    sub.id = uuid()

    -- Get the channels from the subscribe request
    local chans = channels(sub.req)

    for _, chan in ipairs(chans) do
        addsub(sub, chan)
        print("Subscribed to channel:", chan, sub.id)
    end

    return sub
end

function message(pub, sub)
    -- Get the channels from the publish request
    local chans = channels(pub.req)

    for _, chan in ipairs(chans) do
        if issub(sub, chan) then
            -- Deliver the message to the subscriber
            print("Published message to channel subscriber:", chan, sub.id)

            return pub
        end
    end

    -- If the subscriber is not in one of the channels,
    -- don't deliver the message
end

function unsubscribe(sub)
    -- Get the channels from the subscribe request
    local chans = channels(sub.req)

    for _, chan in ipairs(chans) do
        delsub(sub, chan)
        print("Unsubscribed from channel:", chan, sub.id)
    end
end
