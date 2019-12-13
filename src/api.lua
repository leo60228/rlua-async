local api = select(1, ...)

async = {}
async.yield = function()
    coroutine.yield(api:yield())
end
async.sleep = function(duration)
    coroutine.yield(api:sleep(duration))
end
