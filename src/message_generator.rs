use crate::options::Scenario;
use diameter::avp::Avp;
use diameter::avp::AvpType;
use diameter::avp::AvpValue;
use diameter::avp::Enumerated;
use diameter::avp::Identity;
use diameter::avp::OctetString;
use diameter::avp::UTF8String;
use diameter::avp::Unsigned32;
use diameter::dictionary;
use diameter::flags;
use diameter::{ApplicationId, CommandCode, DiameterMessage};

pub struct MessageGenerator {
    command_code: CommandCode,
    application_id: ApplicationId,
    flags: u8,
    seq_num: u32,
    avps: Vec<AvpContainer>,
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
    Variable(String),
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

            let value = if a.value.variable.is_some() {
                AvpValueContainer::Variable(a.value.variable.clone().unwrap())
            } else if a.value.constant.is_some() {
                AvpValueContainer::Variable(a.value.constant.clone().unwrap())
            } else {
                panic!("Both constant and variable for avp {} are None", a.name);
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
            // let variable = avp.value.variable.clone().unwrap();
            let value = match &avp.value {
                AvpValueContainer::Variable(v) => v.clone(),
                AvpValueContainer::Constant(v) => v.to_string(),
            };

            let avp_value: AvpValue = match avp.avp_type {
                AvpType::Identity => Identity::new(&value).into(),
                AvpType::UTF8String => UTF8String::new(&value).into(),
                AvpType::OctetString => OctetString::new(value.clone().into()).into(),
                AvpType::Unsigned32 => Unsigned32::new(value.parse().unwrap()).into(),
                AvpType::Enumerated => Enumerated::new(value.parse().unwrap()).into(),
                _ => todo!(),
            };

            diameter_msg.add_avp(Avp::new(avp.code, avp.vendor_id, avp.flags, avp_value));
        }

        println!("diameter_msg : {}", diameter_msg);

        diameter_msg
    }
}
