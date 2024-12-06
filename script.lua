local log = require "log"
local json = require "json"
local uuid = require "uuid"

function publish(msg)
    -- Add a unique id to the message
    msg.id = uuid()
    log.info(("publish: msg=%s"):format(json(msg)))

    -- Return the message to be published to all subscribers
    return msg
end

function message(msg)
    log.info(("message: msg=%s"):format(json(msg)))

    -- Return the message to be published to this subscriber
    return msg
end
