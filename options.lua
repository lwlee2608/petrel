local option = {
    call_timeout_ms = 1000,
    duration_s = 3,
    call_rate = 1,
    -- call_rate = 150000,
    log_requests = true,
    log_responses = true,
    -- log_requests = false,
    -- log_responses = false,
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
                command = "CreditControl", application = "CreditControl", flags = 0,
                avps = {
                    { name = "Origin-Host", value = "host.example.com" },
                    { name = "Origin-Realm", value = "realm.example.com" },
                    { name = "Product-Name", value = "Petrel" },
                    { name = "Session-Id", value = "ses;${COUNTER}" },
                    { name = "CC-Request-Type", value = "1" },
                    { name = "CC-Request-Number", value = "100" },
                },
            },
        },
    },
}

return option
