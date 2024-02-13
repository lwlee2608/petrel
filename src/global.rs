use crate::options;
use rand::Rng;
use std::cell::RefCell;
use std::collections::HashMap;

pub struct Global {
    variables: HashMap<String, Variable>,
}

impl Global {
    pub fn new(options: &options::Global) -> Self {
        let mut variables = HashMap::new();
        for map in &options.variables {
            for (var_name, value) in map {
                let variable = Variable {
                    name: var_name.clone(),
                    value: match value.func {
                        options::Function::IncrementalCounter => Box::new(IncCounter::new(value)),
                        options::Function::RandomNumber => Box::new(Random::new(value)),
                        _ => todo!("Function not implemented"),
                    },
                };
                variables.insert(var_name.clone(), variable);
            }
        }
        Global { variables }
    }

    pub fn get_variable(&self, name: &str) -> Option<&Variable> {
        self.variables.get(name)
    }
}

pub struct Variable {
    pub name: String,
    pub value: Box<dyn Function>,
}

pub trait Function {
    fn get(&self) -> String;
}

pub struct IncCounter {
    counter: RefCell<i32>,
    max: i32,
    min: i32,
    step: i32,
}

impl IncCounter {
    pub fn new(option: &options::Variable) -> Self {
        IncCounter {
            counter: RefCell::new(option.min),
            max: option.max,
            min: option.min,
            step: option.step,
        }
    }
}

impl Function for IncCounter {
    fn get(&self) -> String {
        let value = *self.counter.borrow();
        *self.counter.borrow_mut() += self.step;
        if *self.counter.borrow() > self.max {
            *self.counter.borrow_mut() = self.min;
        }
        value.to_string()
    }
}

pub struct Random {
    min: i32,
    max: i32,
}

impl Random {
    pub fn new(option: &options::Variable) -> Self {
        Random {
            min: option.min,
            max: option.max,
        }
    }
}

impl Function for Random {
    fn get(&self) -> String {
        let mut rng = rand::thread_rng();
        let value = rng.gen_range(self.min..=self.max);
        value.to_string()
    }
}
