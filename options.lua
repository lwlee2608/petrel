local option = {
    call_timeout_ms = 1000,
    duration_s = 3,
    call_rate = 1,
    -- call_rate = 150000,
    log_requests = true,
    log_responses = true,
    -- log_requests = false,
    -- log_responses = false,

    -- https://gull.sourceforge.net/doc/diameter.html
    scenarios = {
        {
            name = "CER",
            message = {
                command = "Capability-Exchange", application = "Common", flags = 0,
                avps = {
                    { name = "Origin-Host", value = "host.example.com" },
                    { name = "Origin-Realm", value = "realm.example.com" },
                    { name = "Session-Id", value = "ses;2345888" },
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
                    { name = "Session-Id", value = "ses;2345888" },
                    { name = "CC-Request-Type", value = "1" },
                    { name = "CC-Request-Number", value = "100" },
                },
            },
        },
    },
}

return option
