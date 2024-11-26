use crate::global;
use crate::global::Global;
use crate::options;
use chrono::DateTime;
use chrono::Utc;
use diameter::avp::Address;
use diameter::avp::AvpType;
use diameter::avp::AvpValue;
use diameter::avp::Enumerated;
use diameter::avp::Grouped;
use diameter::avp::IPv4;
use diameter::avp::IPv6;
use diameter::avp::Identity;
use diameter::avp::OctetString;
use diameter::avp::Time;
use diameter::avp::UTF8String;
use diameter::avp::Unsigned32;
use diameter::avp::Unsigned64;
use diameter::dictionary::Dictionary;
use diameter::flags;
use diameter::{ApplicationId, CommandCode, DiameterMessage};
use regex::Regex;
use std::error::Error;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::result::Result;
use std::sync::Arc;
use uuid::Uuid;

pub struct Scenario<'a> {
    name: String,
    message: Message<'a>,
}

impl<'a> Scenario<'a> {
    pub fn new(
        options: &options::Scenario,
        global: &'a Global,
        dict: Arc<Dictionary>,
    ) -> Result<Self, Box<dyn Error>> {
        return Ok(Scenario {
            name: options.name.clone(),
            message: Message::new(options, global, dict)?,
        });
    }

    pub fn next_message(&mut self) -> Result<DiameterMessage, Box<dyn Error>> {
        self.message.message()
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

pub struct Message<'a> {
    command_code: CommandCode,
    application_id: ApplicationId,
    flags: u8,
    seq_num: u32,
    avps: Vec<Avp<'a>>,
    dict: Arc<Dictionary>,
}

impl<'a> Message<'a> {
    pub fn new(
        scenario: &options::Scenario,
        global: &'a Global,
        dict: Arc<Dictionary>,
    ) -> Result<Self, Box<dyn Error>> {

        let command_code = dict
            .get_command_code_by_name(&scenario.message.command)
            .ok_or(format!(
                "Unknown Command-Code '{}'",
                scenario.message.command
            ))?;

        let application_id = dict
            .get_application_id_by_name(&scenario.message.application)
            .ok_or(format!(
                "Unknown Application-Id '{}'",
                scenario.message.application
            ))?;

        let flags = flags::REQUEST;

        let mut avps = vec![];

        for a in &scenario.message.avps {
            let avp_definition = dict
                .get_avp_by_name(&a.name)
                .ok_or(format!("AVP '{}' not found in dictionary", a.name))?;

            let value = Value::new(&a.value, avp_definition.avp_type, global, Arc::clone(&dict))
                .map_err(|e| format!("AVP '{}', error: {}", avp_definition.name, e.to_string()))?;

            let avp_flags = if avp_definition.m_flag {
                diameter::avp::flags::M
            } else {
                0
            };
            let avp = Avp {
                code: avp_definition.code,
                vendor_id: avp_definition.vendor_id,
                flags: avp_flags,
                value,
            };
            avps.push(avp);
        }

        Ok(Message {
            command_code,
            application_id,
            flags,
            seq_num: 0,
            avps,
            dict,
        })
    }

    pub fn message(&mut self) -> Result<DiameterMessage, Box<dyn Error>> {
        self.seq_num += 1;
        // TODO remove this
        let seq_num = Uuid::new_v4().as_u128() as u32;

        let mut diameter_msg = DiameterMessage::new(
            self.command_code,
            self.application_id,
            self.flags,
            seq_num,
            seq_num,
            // self.seq_num,
            // self.seq_num,
            Arc::clone(&self.dict),
        );

        for avp in &self.avps {
            let value = avp.value.get_value()?;
            diameter_msg.add_avp(avp.code, avp.vendor_id, avp.flags, value);
        }

        Ok(diameter_msg)
    }
}

pub fn string_to_avp_value(
    str: &str,
    avp_type: diameter::avp::AvpType,
) -> Result<AvpValue, Box<dyn Error>> {
    let value = match avp_type {
        AvpType::Address => {
            let addr: IpAddr = str.parse().expect("Invalid IP address");
            match addr {
                IpAddr::V4(addr) => Address::from_ipv4(addr).into(),
                IpAddr::V6(addr) => Address::from_ipv6(addr).into(),
            }
        }
        AvpType::AddressIPv4 => {
            let addr: Ipv4Addr = str.parse().expect("Invalid IPv4 address");
            IPv4::new(addr).into()
        }
        AvpType::AddressIPv6 => {
            let addr: Ipv6Addr = str.parse().expect("Invalid IPv6 address");
            IPv6::new(addr).into()
        }
        AvpType::Identity => Identity::new(&str).into(),
        AvpType::DiameterURI => UTF8String::new(&str).into(),
        AvpType::Enumerated => Enumerated::new(str.parse()?).into(),
        AvpType::Float32 => Unsigned32::new(str.parse()?).into(),
        AvpType::Float64 => Unsigned64::new(str.parse()?).into(),
        AvpType::Integer32 => Unsigned32::new(str.parse()?).into(),
        AvpType::Integer64 => Unsigned64::new(str.parse()?).into(),
        AvpType::OctetString => {
            // TODO
            OctetString::new(str.as_bytes().to_vec()).into()
        }
        AvpType::Unsigned32 => Unsigned32::new(str.parse()?).into(),
        AvpType::Unsigned64 => Unsigned64::new(str.parse()?).into(),
        AvpType::UTF8String => UTF8String::new(&str).into(),
        AvpType::Time => {
            let time = str.parse::<DateTime<Utc>>()?;
            Time::new(time).into()
        }
        AvpType::Grouped => return Err("Invalid Grouped AVP value".into()),
        AvpType::Unknown => return Err("Unknown AVP type".into()),
    };
    Ok(value)
}

struct Avp<'a> {
    code: u32,
    vendor_id: Option<u32>,
    flags: u8,
    value: Value<'a>,
}

struct Value<'a> {
    source: String,
    avp_type: diameter::avp::AvpType,
    variables: Vec<&'a global::Variable>,
    constant: Option<AvpValue>,
}

impl<'a> Value<'a> {
    pub fn new(
        source: &options::Value,
        avp_type: diameter::avp::AvpType,
        global: &'a Global,
        dict: Arc<Dictionary>,
    ) -> Result<Self, Box<dyn Error>> {
        match source {
            options::Value::String(source) => {
                // Scan for variables
                let variable_pattern = Regex::new(r"\$\{([^}]+)\}")?;
                let mut variables = vec![];
                for caps in variable_pattern.captures_iter(source) {
                    let cap = caps[1].to_string();
                    let var = global.get_variable(&cap).unwrap();
                    variables.push(var);
                }

                // If no variable found, make this a constant
                let constant = if variables.is_empty() {
                    let value = string_to_avp_value(source, avp_type)?;
                    Some(value)
                } else {
                    None
                };

                Ok(Value {
                    source: source.into(),
                    avp_type,
                    variables,
                    constant,
                })
            }
            options::Value::Avp(source) => {
                let variables = vec![];
                if avp_type != AvpType::Grouped {
                    return Err("Invalid AVP type for AVP value".into());
                }
                let mut avps = vec![];
                for a in source {

                    let avp_definition = dict
                        .get_avp_by_name(&a.name)
                        .ok_or(format!("AVP '{}' not found in dictionary", a.name))?;

                    let value =
                        Value::new(&a.value, avp_definition.avp_type, global, Arc::clone(&dict))?;
                    let value = value.get_value()?;
                    let avp = diameter::avp::Avp::new(
                        avp_definition.code,
                        avp_definition.vendor_id,
                        0,
                        value,
                        Arc::clone(&dict),
                    );

                    avps.push(avp);
                }
                let value: AvpValue = Grouped::new(avps, dict).into();
                let constant = Some(value);
                Ok(Value {
                    source: "TODO".into(),
                    avp_type,
                    variables,
                    constant,
                })
            }
        }
    }

    // TODO Rename
    fn compute(&self) -> String {
        let mut result: String = self.source.clone();
        for v in &self.variables {
            let counter = v.value.get();
            let name = &v.name;
            result = result.replace(&format!("${{{}}}", name), &counter);
        }
        result
    }

    pub fn get_value(&self) -> Result<AvpValue, Box<dyn Error>> {
        match &self.constant {
            Some(v) => Ok(v.clone()),
            None => Ok(string_to_avp_value(&self.compute(), self.avp_type)?),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options;

    #[test]
    fn test_constant() {
        let dict = Dictionary::new(&vec![]);
        let dict = Arc::new(dict);

        let global = Global::new(&options::Global {
            variables: vec![std::iter::once((
                "COUNTER".into(),
                options::Variable {
                    func: options::Function::IncrementalCounter,
                    min: 1,
                    max: 5,
                    step: 3,
                },
            ))
            .collect()],
        });

        let variable = Value::new(
            &options::Value::String("example.origin.host".into()),
            AvpType::UTF8String,
            &global,
            dict,
        )
        .unwrap();

        assert_eq!("example.origin.host", variable.compute());
        assert_eq!("example.origin.host", variable.compute());
        assert_eq!("example.origin.host", variable.compute());
    }

    #[test]
    fn test_counter_variable() {
        let dict = Dictionary::new(&vec![]);
        let dict = Arc::new(dict);

        let global = Global::new(&options::Global {
            variables: vec![std::iter::once((
                "COUNTER".into(),
                options::Variable {
                    func: options::Function::IncrementalCounter,
                    min: 1,
                    max: 5,
                    step: 3,
                },
            ))
            .collect()],
        });

        let variable = Value::new(
            &options::Value::String("ses;${COUNTER}".into()),
            AvpType::UTF8String,
            &global,
            dict,
        )
        .unwrap();

        assert_eq!("ses;1", variable.compute());
        assert_eq!("ses;4", variable.compute());
        assert_eq!("ses;1", variable.compute());
    }

    #[test]
    fn test_2_counters_variable() {
        let dict = Dictionary::new(&vec![]);
        let dict = Arc::new(dict);

        let global = Global::new(&options::Global {
            variables: vec![
                std::iter::once((
                    "COUNTER1".into(),
                    options::Variable {
                        func: options::Function::IncrementalCounter,
                        min: 0,
                        max: 5,
                        step: 1,
                    },
                ))
                .collect(),
                std::iter::once((
                    "COUNTER2".into(),
                    options::Variable {
                        func: options::Function::IncrementalCounter,
                        min: 1,
                        max: 5,
                        step: 3,
                    },
                ))
                .collect(),
            ],
        });

        let variable = Value::new(
            &options::Value::String("ses;${COUNTER1}_${COUNTER2}".into()),
            AvpType::UTF8String,
            &global,
            dict,
        )
        .unwrap();

        assert_eq!("ses;0_1", variable.compute());
        assert_eq!("ses;1_4", variable.compute());
        assert_eq!("ses;2_1", variable.compute());
    }
}
