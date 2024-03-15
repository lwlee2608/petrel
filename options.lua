local option = {
    parallel = 1,
    call_timeout_ms = 1000,
    duration_s = 2,
    call_rate = 1,
    log_requests = true,
    log_responses = true,
    -- duration_s = 15,
    -- call_rate = 100000,
    -- log_requests = false,
    -- log_responses = false,
    protocol = "Diameter",
    globals = {
        variables = {
            {
                COUNTER = {
                    func = "incremental_counter",
                    min = 100000000,
                    max = 900000000,
                    step = 10,
                },
            },
            {
                RANDOM = {
                    func = "random_number",
                    min = 1000,
                    max = 9999,
                    step = 1,
                }
            },
        },
    },
    -- https://gull.sourceforge.net/doc/diameter.html
    scenarios = {
        {
            name = "CER",
            type = "Init",
            message = {
                command = "Capabilities-Exchange", application = "Base", flags = 0,
                avps = {
                    { name = "Origin-Host", value = "host.example.com" },
                    { name = "Origin-Realm", value = "realm.example.com" },
                    { name = "Destination-Realm", value = "dest.realm.org" },
                    { name = "Host-IP-Address", value = "::1" },
                    { name = "Auth-Application-Id", value = "4" },
                    { name = "Product-Name", value = "Petrel" },
                },
            },
        },
        {
            name = "CCR",
            type = "Repeating",
            message = {
                command = "Credit-Control", application = "Charging Control", flags = 0,
                avps = {
                    { name = "Origin-Host", value = "host.example.com" },
                    { name = "Origin-Realm", value = "realm.example.com" },
                    { name = "Product-Name", value = "Petrel" },
                    { name = "Session-Id", value = "ses;${COUNTER}_${RANDOM}" },
                    { name = "CC-Request-Type", value = "1" },
                    { name = "CC-Request-Number", value = "100" },
                    -- { name = "Subscription-Id",
                    --     value = {
                    --         { name = "Subscription-Id-Type", value = "1" },
                    --         { name = "Subscription-Id-Data", value = "subs-data" },
                    --     },
                    -- },
                },
            },
        },
    },
}

return option
