use std::collections::HashMap;

use mlua::prelude::LuaSerdeExt;
use mlua::UserData;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Options {
    pub parallel: u32,
    pub call_rate: u32,
    pub call_timeout_ms: u32,
    pub duration_s: u32,
    pub log_requests: bool,
    pub log_responses: bool,
    pub globals: Global,
    pub protocol: Protocol,
    pub scenarios: Vec<Scenario>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Protocol {
    Diameter,
    HTTP2,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Scenario {
    pub name: String,
    #[serde(rename = "type")]
    pub scenario_type: ScenarioType,
    pub message: Message,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum ScenarioType {
    Once,
    Repeating,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Message {
    pub command: String,
    pub application: String,
    pub avps: Vec<Avp>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Avp {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Global {
    pub variables: Vec<HashMap<String, Variable>>,
}

// TODO Different function should have different fields
// eg. random_number should not have 'step' field
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Variable {
    pub func: Function,
    pub min: i32,
    pub max: i32,
    pub step: i32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
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
                    parallel = 4,
                    call_rate = 20000, 
                    call_timeout_ms = 1000, 
                    duration_s = 60, 
                    log_requests = false,
                    log_responses = false,
                    protocol = "Diameter",
                    globals = {
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
                    },
                    scenarios = {
                        {
                            name = "CER",
                            type = "Once",
                            message = {
                                command = "Capability-Exchange", application = "Common", flags = 0,
                                avps = {
                                    { name = "Origin-Host", value = "host.example.com" },
                                    { name = "Origin-Realm", value = "realm.example.com" },
                                },
                            },
                        },
                    },
                }"#,
            )
            .eval()?;

        let options: Options = lua.from_value(value)?;

        assert_eq!(options.parallel, 4);
        assert_eq!(options.call_rate, 20000);
        assert_eq!(options.call_timeout_ms, 1000);
        assert_eq!(options.duration_s, 60);
        assert_eq!(options.log_requests, false);
        assert_eq!(options.log_responses, false);
        assert_eq!(options.globals.variables.len(), 1);
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
        assert_eq!(options.globals.variables[0], expected_variables);
        assert_eq!(options.scenarios.len(), 1);
        assert_eq!(
            options.scenarios[0],
            Scenario {
                name: "CER".into(),
                scenario_type: ScenarioType::Once,
                message: Message {
                    command: "Capability-Exchange".into(),
                    application: "Common".into(),
                    avps: vec![
                        Avp {
                            name: "Origin-Host".into(),
                            value: "host.example.com".into(),
                        },
                        Avp {
                            name: "Origin-Realm".into(),
                            value: "realm.example.com".into(),
                        },
                    ],
                },
            },
        );

        Ok(())
    }
}
