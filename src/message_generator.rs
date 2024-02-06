use crate::options::Scenario;
use diameter::avp::Address;
use diameter::avp::Avp;
use diameter::avp::AvpType;
use diameter::avp::AvpValue;
use diameter::avp::Enumerated;
use diameter::avp::Identity;
use diameter::avp::OctetString;
use diameter::avp::UTF8String;
use diameter::avp::Unsigned32;
use diameter::avp::Unsigned64;
use diameter::dictionary;
use diameter::flags;
use diameter::{ApplicationId, CommandCode, DiameterMessage};
use regex::Regex;
use std::cell::RefCell;

pub struct MessageGenerator {
    command_code: CommandCode,
    application_id: ApplicationId,
    flags: u8,
    seq_num: u32,
    avps: Vec<AvpContainer>,
}

impl MessageGenerator {
    pub fn new(scenario: &Scenario) -> Self {
        // TODO
        // let command_code = CommandCode::from(scenario.message.command.as_str());
        // let application_id = ApplicationId::from(scenario.message.application.as_str());
        let command_code = CommandCode::CreditControl;
        let application_id = ApplicationId::CreditControl;
        let flags = flags::REQUEST;

        let mut avps = vec![];

        for a in &scenario.message.avps {
            let avp_definition = dictionary::DEFAULT_DICT.get_avp_by_name(&a.name).unwrap();

            let value = match &a.value.variable {
                Some(v) => AvpValueContainer::Variable(AvpVariableValue::new(v)),
                None => match &a.value.constant {
                    Some(c) => {
                        let v = string_to_avp_value(c, avp_definition.avp_type);
                        AvpValueContainer::Constant(v)
                    }
                    None => panic!("Both constant and variable for avp {} are None", a.name),
                },
            };

            let avp = AvpContainer {
                code: avp_definition.code,
                vendor_id: None, // TODO avp_definition.vendor_id,
                flags: 0,        // TODO avp_definition.flags,
                avp_type: avp_definition.avp_type,
                value,
            };

            avps.push(avp);
        }

        MessageGenerator {
            command_code,
            application_id,
            flags,
            seq_num: 0,
            avps,
        }
    }

    pub fn message(&mut self) -> DiameterMessage {
        self.seq_num += 1;
        let mut diameter_msg = DiameterMessage::new(
            self.command_code,
            self.application_id,
            self.flags,
            self.seq_num,
            self.seq_num,
        );

        for avp in &self.avps {
            let value: AvpValue = match &avp.value {
                AvpValueContainer::Constant(v) => v.clone(),
                AvpValueContainer::Variable(v) => v.get_value(avp.avp_type),
            };
            diameter_msg.add_avp(Avp::new(avp.code, avp.vendor_id, avp.flags, value));
        }

        println!("diameter_msg : {}", diameter_msg);

        diameter_msg
    }
}

pub fn string_to_avp_value(str: &str, avp_type: AvpType) -> AvpValue {
    match avp_type {
        AvpType::Address => Address::new(str.as_bytes().to_vec()).into(),
        AvpType::AddressIPv4 => Address::new(str.as_bytes().to_vec()).into(),
        AvpType::AddressIPv6 => Address::new(str.as_bytes().to_vec()).into(),
        AvpType::Identity => Identity::new(&str).into(),
        AvpType::DiameterURI => UTF8String::new(&str).into(),
        AvpType::Enumerated => Enumerated::new(str.parse().unwrap()).into(),
        AvpType::Float32 => Unsigned32::new(str.parse().unwrap()).into(),
        AvpType::Float64 => Unsigned64::new(str.parse().unwrap()).into(),
        AvpType::Grouped => todo!(),
        AvpType::Integer32 => Unsigned32::new(str.parse().unwrap()).into(),
        AvpType::Integer64 => Unsigned64::new(str.parse().unwrap()).into(),
        AvpType::OctetString => OctetString::new(str.as_bytes().to_vec()).into(),
        AvpType::Unsigned32 => Unsigned32::new(str.parse().unwrap()).into(),
        AvpType::Unsigned64 => Unsigned64::new(str.parse().unwrap()).into(),
        AvpType::UTF8String => UTF8String::new(&str).into(),
        AvpType::Time => Unsigned32::new(str.parse().unwrap()).into(),
        AvpType::Unknown => todo!("Throw Error"),
    }
}

struct AvpContainer {
    code: u32,
    vendor_id: Option<u32>,
    flags: u8,
    avp_type: AvpType,
    value: AvpValueContainer,
}

enum AvpValueContainer {
    Constant(AvpValue),
    Variable(AvpVariableValue),
}

struct AvpVariableValue {
    source: String,
    functions: Vec<Box<dyn Function>>,
}

impl AvpVariableValue {
    pub fn new(source: &str) -> Self {
        let variable_pattern = Regex::new(r"\$\{([^}]+)\}").unwrap();
        let mut functions: Vec<Box<dyn Function>> = Vec::new();

        for caps in variable_pattern.captures_iter(source) {
            let cap = caps[1].to_string();
            if cap == "COUNTER" {
                functions.push(Box::new(CounterFunction::new()));
                continue;
            }
        }

        AvpVariableValue {
            source: source.into(),
            functions,
        }
    }

    pub fn execute(&self) -> String {
        let function = &self.functions[0];
        let counter = function.execute();
        let name = function.name();
        let result = self.source.replace(&format!("${{{}}}", name), &counter);
        result
    }

    pub fn get_value(&self, avp_type: AvpType) -> AvpValue {
        let value = self.execute();
        string_to_avp_value(&value, avp_type)
    }
}

pub trait Function {
    fn execute(&self) -> String;
    fn name(&self) -> String;
}

struct CounterFunction {
    name: String,
    counter: RefCell<i32>, // Hopefully this doesn't explode
}

impl CounterFunction {
    pub fn new() -> Self {
        CounterFunction {
            name: "COUNTER".to_string(),
            counter: RefCell::new(0),
        }
    }
}

impl Function for CounterFunction {
    fn execute(&self) -> String {
        *self.counter.borrow_mut() += 1;
        self.counter.borrow().to_string()
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable() {
        let variable = AvpVariableValue::new("ses;${COUNTER}");

        assert_eq!("ses;1", variable.execute());
        assert_eq!("ses;2", variable.execute());
        assert_eq!("ses;3", variable.execute());
    }
}
