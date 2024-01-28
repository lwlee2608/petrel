local option = {
    -- call_rate = 50000,
    call_rate = 1,
    call_timeout_ms = 1000,
    duration_s = 10,
    log_requests = true,
    log_responses = true,
    scenarios = {
        {
            name = "CER",
            message = {
                command = "CER", application = "Common", flags = 0,
                avps = {
                    { code = 264, type = "identity",   value = "host.example.com", mandatory = true },
                    { code = 296, type = "identity",   value = "realm.example.com",mandatory =  true },
                    { code = 263, type = "utf8string", value = "ses;2345888", mandatory = true },
                    { code = 415, type = "unsigned32", value = 2001, mandatory = false },
                    { code = 416, type = "enumerated", value = 1, mandatory = true },
                    { code = 415, type = "unsigned32", value = 1000, mandatory = true },
                },
            },
        },
        {
            name = "CCR",
            message = {
                command = "CreditControl", application = "CreditControl", flags = 0,
                avps = {
                    { code = 264, type = "identity",   value = "host.example.com", mandatory = true },
                    { code = 296, type = "identity",   value = "realm.example.com",mandatory =  true },
                    { code = 263, type = "utf8string", value = "ses;2345888", mandatory = true },
                    { code = 415, type = "unsigned32", value = 2001, mandatory = false },
                    { code = 416, type = "enumerated", value = 1, mandatory = true },
                    { code = 415, type = "unsigned32", value = 1000, mandatory = true },
                },
            },
        },
    }
}

return option
