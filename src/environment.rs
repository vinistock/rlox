use std::collections::HashMap;

use crate::vm::{RuntimeError, Value};

pub struct Environment {
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Result<&Value, RuntimeError> {
        match self.values.get(name) {
            Some(value) => Ok(value),
            None => Err(RuntimeError::UndefinedVariable(format!(
                "{} variable is not defined",
                name
            ))),
        }
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), RuntimeError> {
        if let Some(v) = self.values.get_mut(name) {
            *v = value;
            Ok(())
        } else {
            Err(RuntimeError::UndefinedVariable(format!(
                "{} variable is not defined",
                name
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defining_global_variables() {
        let mut env = Environment::new();
        env.define("x".to_string(), Value::Number(42.0));
        env.define("y".to_string(), Value::String("Hello".to_string()));

        assert_eq!(env.get("x").unwrap(), &Value::Number(42.0));
        assert_eq!(env.get("y").unwrap(), &Value::String("Hello".to_string()));
    }
}
