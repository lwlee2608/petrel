use std::collections::HashMap;

use mlua::prelude::LuaSerdeExt;
use mlua::UserData;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Options {
    pub call_rate: u32,
    pub call_timeout_ms: u32,
    pub duration_s: u32,
    pub log_requests: bool,
    pub log_responses: bool,
    pub variables: Vec<HashMap<String, Variable>>,
    pub scenarios: Vec<Scenario>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Scenario {
    pub name: String,
    pub message: Message,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Message {
    pub command: String,
    pub application: String,
    pub avps: Vec<Avp>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Avp {
    pub name: String,
    pub value: Value,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Value {
    pub constant: Option<String>,
    pub variable: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Variable {
    pub func: Function,
    pub min: u32,
    pub max: u32,
    pub step: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Function {
    IncrementalCounter,
    RandomNumber,
    CustomScript,
}

impl UserData for Options {}

pub fn load(filename: &str) -> Options {
    let lua = mlua::Lua::new();
    let lua_script = std::fs::read_to_string(filename).expect("Failed to read options file");
    let value = lua
        .load(&lua_script)
        .eval()
        .expect("Failed to load options");
    let options: Options = lua.from_value(value).expect("Failed to convert options");
    options
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_options() -> mlua::Result<()> {
        let lua = mlua::Lua::new();
        let value = lua
            .load(
                r#"{ 
                    call_rate = 20000, 
                    call_timeout_ms = 1000, 
                    duration_s = 60, 
                    log_requests = false,
                    log_responses = false,
                    variables = {
                        {
                            COUNTER = {
                                func = "incremental_counter",
                                min = 1,
                                max = 1000000000,
                                step = 1,
                            },
                        },
                    },
                    scenarios = {
                        {
                            name = "CER",
                            message = {
                                command = "Capability-Exchange", application = "Common", flags = 0,
                                avps = {
                                    { name = "Origin-Host", value = { constant = "host.example.com" } },
                                    { name = "Origin-Realm", value = { constant = "realm.example.com" } },
                                },
                            },
                        },
                    },
                }"#,
            )
            .eval()?;

        let options: Options = lua.from_value(value)?;

        assert_eq!(options.call_rate, 20000);
        assert_eq!(options.call_timeout_ms, 1000);
        assert_eq!(options.duration_s, 60);
        assert_eq!(options.log_requests, false);
        assert_eq!(options.log_responses, false);
        assert_eq!(options.variables.len(), 1);
        let expected_variables: HashMap<String, Variable> = [(
            "COUNTER".to_string(),
            Variable {
                func: Function::IncrementalCounter,
                min: 1,
                max: 1000000000,
                step: 1,
            },
        )]
        .into();
        assert_eq!(options.variables[0], expected_variables);
        assert_eq!(options.scenarios.len(), 1);
        assert_eq!(
            options.scenarios[0],
            Scenario {
                name: "CER".into(),
                message: Message {
                    command: "Capability-Exchange".into(),
                    application: "Common".into(),
                    avps: vec![
                        Avp {
                            name: "Origin-Host".into(),
                            value: Value {
                                constant: Some("host.example.com".into()),
                                variable: None,
                            },
                        },
                        Avp {
                            name: "Origin-Realm".into(),
                            value: Value {
                                constant: Some("realm.example.com".into()),
                                variable: None,
                            },
                        },
                    ],
                },
            },
        );

        Ok(())
    }
}
