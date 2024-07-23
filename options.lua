local option = {
    parallel = 4,
    call_timeout = "1000ms",
    -- duration = "2s",
    -- call_rate = 1,
    -- log_requests = true,
    -- log_responses = true,
    duration = "15s",
    call_rate = 90000,
    log_requests = false,
    log_responses = false,
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
                CC_REQUEST_NUMBER = {
                    func = "incremental_counter",
                    min = 1,
                    max = 900000000,
                    step = 1,
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
    dictionaries = {
        "https://raw.githubusercontent.com/lwlee2608/diameter-rs/master/dict/3gpp-ro-rf.xml",
    },
    -- Reference: https://gull.sourceforge.net/doc/diameter.html
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
                    { name = "Vendor-Id", value = "12345" },
                },
            },
        },
        {
            name = "Ro-CCR-I",
            type = "Repeating",
            message = {
                command = "Credit-Control", application = "Charging Control", flags = 0,
                avps = {
                    { name = "Origin-Host", value = "host.example.com" },
                    { name = "Origin-Realm", value = "realm.example.com" },
                    { name = "Product-Name", value = "Petrel" },
                    { name = "Session-Id", value = "ses;${COUNTER}_${RANDOM}" },
                    { name = "Auth-Application-Id", value = "4" },
                    { name = "Service-Identifier", value = "1003" },
                    { name = "Service-Context-Id", value = "1003" },
                    { name = "CC-Request-Type", value = "1" },
                    { name = "CC-Request-Number", value = "${CC_REQUEST_NUMBER}" },
                    { name = "Event-Timestamp", value = "2020-01-01T00:00:00Z" },
                    { name = "Subscription-Id",
                        value = {
                            { name = "Subscription-Id-Type", value = "1" },
                            { name = "Subscription-Id-Data", value = "10133100" },
                        },
                    },
                    { name = "Service-Information",
                        value = {
                            { name = "PS-Information",
                                value = {
                                    { name = "Called-Station-Id", value = "10999" },
                                    -- { name = "TGPP-MS-TimeZone", value = "0" },
                                },
                            }
                        },
                    },
                    { name = "Multiple-Services-Indicator", value = "1" },
                    { name = "Multiple-Services-Credit-Control",
                        value = {
                            { name = "Rating-Group", value = "1003" },
                            { name = "Requested-Service-Unit",
                                value = {
                                    { name = "CC-Total-Octets", value = "1048576" },
                                },
                            },
                        },
                    },
                    -- { name = "SGSN-Address", value = "127.0.0.1" },
                    { name = "Requested-Action", value = "0" },
                },
            },
        },
        {
            name = "Ro-CCR-T",
            type = "Repeating",
            message = {
                command = "Credit-Control", application = "Charging Control", flags = 0,
                avps = {
                    { name = "Origin-Host", value = "host.example.com" },
                    { name = "Origin-Realm", value = "realm.example.com" },
                    { name = "Product-Name", value = "Petrel" },
                    { name = "Session-Id", value = "ses;${COUNTER}_${RANDOM}" },
                    { name = "Auth-Application-Id", value = "4" },
                    { name = "Service-Identifier", value = "1003" },
                    { name = "Service-Context-Id", value = "1003" },
                    { name = "CC-Request-Type", value = "3" },
                    { name = "CC-Request-Number", value = "${CC_REQUEST_NUMBER}" },
                    { name = "Event-Timestamp", value = "2020-01-01T00:00:00Z" },
                    { name = "Subscription-Id",
                        value = {
                            { name = "Subscription-Id-Type", value = "1" },
                            { name = "Subscription-Id-Data", value = "10133100" },
                        },
                    },
                    { name = "Service-Information",
                        value = {
                            { name = "PS-Information",
                                value = {
                                    { name = "Called-Station-Id", value = "10999" },
                                    -- { name = "TGPP-MS-TimeZone", value = "0" },
                                },
                            }
                        },
                    },
                    { name = "Multiple-Services-Indicator", value = "1" },
                    { name = "Multiple-Services-Credit-Control",
                        value = {
                            { name = "Rating-Group", value = "1003" },
                            { name = "Requested-Service-Unit",
                                value = {
                                    { name = "CC-Total-Octets", value = "1048576" },
                                },
                            },
                        },
                    },
                    { name = "SGSN-Address", value = "127.0.0.1" },
                    { name = "Requested-Action", value = "0" },
                },
            },
        },
    },
}

return option
