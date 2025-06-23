use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::vm::{RuntimeError, Value};

pub type Env = Rc<RefCell<Environment>>;
pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<Env>,
}

impl Environment {
    pub fn new(enclosing: Option<Env>) -> Self {
        Environment {
            values: HashMap::new(),
            enclosing,
        }
    }

    pub fn new_global() -> Env {
        Rc::new(RefCell::new(Environment::new(None)))
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Result<Value, RuntimeError> {
        match self.values.get(name) {
            Some(value) => Ok(value.clone()),
            None => match self.enclosing {
                Some(ref enclosing) => enclosing.borrow().get(name),
                None => Err(RuntimeError::UndefinedVariable(format!(
                    "{} variable is not defined",
                    name
                ))),
            },
        }
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), RuntimeError> {
        if let Some(v) = self.values.get_mut(name) {
            *v = value;
            Ok(())
        } else {
            match self.enclosing {
                Some(ref mut enclosing) => enclosing.borrow_mut().assign(name, value),
                None => Err(RuntimeError::UndefinedVariable(format!(
                    "{} variable is not defined",
                    name
                ))),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defining_global_variables() {
        let mut env = Environment::new(None);
        env.define("x".to_string(), Value::Number(42.0));
        env.define("y".to_string(), Value::String("Hello".to_string()));

        assert_eq!(env.get("x").unwrap(), Value::Number(42.0));
        assert_eq!(env.get("y").unwrap(), Value::String("Hello".to_string()));
    }

    #[test]
    fn test_getting_variables_from_enclosing_environments() {
        let mut parent_env = Environment::new(None);
        parent_env.define("x".to_string(), Value::Number(42.0));
        parent_env.define("y".to_string(), Value::String("Hello".to_string()));

        let child_env = Environment::new(Some(Rc::new(RefCell::new(parent_env))));

        assert_eq!(child_env.get("x").unwrap(), Value::Number(42.0));
        assert_eq!(child_env.get("y").unwrap(), Value::String("Hello".to_string()));
        assert!(child_env.get("z").is_err());
    }

    #[test]
    fn test_assigning_variables_for_enclosing_environments() {
        let mut parent_env = Environment::new(None);
        parent_env.define("x".to_string(), Value::Number(42.0));
        parent_env.define("y".to_string(), Value::String("Hello".to_string()));

        let mut child_env = Environment::new(Some(Rc::new(RefCell::new(parent_env))));
        child_env.assign("x", Value::Number(100.0)).unwrap();

        assert_eq!(child_env.get("x").unwrap(), Value::Number(100.0));
        assert_eq!(child_env.get("y").unwrap(), Value::String("Hello".to_string()));
        assert!(child_env.get("z").is_err());
    }
}
