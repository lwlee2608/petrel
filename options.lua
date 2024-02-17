local option = {
    call_timeout_ms = 1000,
    -- duration_s = 5,
    -- call_rate = 1,
    -- log_requests = true,
    -- log_responses = true,
    duration_s = 15,
    call_rate = 150000,
    log_requests = false,
    log_responses = false,
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
            message = {
                command = "Capability-Exchange", application = "Common", flags = 0,
                avps = {
                    { name = "Origin-Host", value = "host.example.com" },
                    { name = "Origin-Realm", value = "realm.example.com" },
                    { name = "CC-Correlation-Id", value = "ses;2345888" },
                },
            },
        },
        {
            name = "CCR",
            message = {
                command = "Credit-Control", application = "Charging Control", flags = 0,
                avps = {
                    { name = "Origin-Host", value = "host.example.com" },
                    { name = "Origin-Realm", value = "realm.example.com" },
                    { name = "Product-Name", value = "Petrel" },
                    { name = "Session-Id", value = "ses;${COUNTER}_${RANDOM}" },
                    { name = "CC-Request-Type", value = "1" },
                    { name = "CC-Request-Number", value = "100" },
                },
            },
        },
    },
}

return option
