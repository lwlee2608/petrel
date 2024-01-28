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
    pub scenarios: Vec<Scenario>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Scenario {
    pub name: String,
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
                    scenarios = {
                        { name = "scenario1" },
                        { name = "scenario2" },
                    }
                }"#,
            )
            .eval()?;

        let options: Options = lua.from_value(value)?;

        assert_eq!(options.call_rate, 20000);
        assert_eq!(options.call_timeout_ms, 1000);
        assert_eq!(options.duration_s, 60);
        assert_eq!(options.log_requests, false);
        assert_eq!(options.log_responses, false);
        assert_eq!(options.scenarios.len(), 2);
        assert_eq!(options.scenarios[0].name, "scenario1");
        assert_eq!(options.scenarios[1].name, "scenario2");

        Ok(())
    }
}
