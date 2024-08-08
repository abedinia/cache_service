counter = 0
key = "exampleKey"

request = function()
    counter = counter + 1
    if counter % 10 < 3 then
        return wrk.format("POST", "/cache", {["Content-Type"] = "application/json"}, '{"key":"'..key..'","data":"exampleData","ttl":60}')
    else
        return wrk.format("GET", "/cache/" .. key)
    end
end
