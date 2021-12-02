use std::borrow::Borrow;
use std::collections::HashMap;

use crate::ast::*;

#[derive(Debug)]
pub struct Environment {
    variables: HashMap<String, Vec<RefVal>>,
    stack: Vec<RefVal>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            variables: HashMap::new(),
            stack: Vec::new(),
        }
    }

    pub fn pop_stack(&mut self) -> RefVal {
        self.stack.pop().unwrap()
    }

    pub fn push_stack(&mut self, val: RefVal) {
        self.stack.push(val);
    }

    pub fn register_external_fun(
        &mut self,
        name: &'static str,
        arity: usize,
        ptr: fn(&mut Environment) -> Result<RefVal, String>,
    ) {
        self.variables.insert(
            name.to_string(),
            vec![RefVal::owned(Value::Function(Function::Lib {
                name,
                arity,
                ptr,
            }))],
        );
    }

    pub fn bind_var(&mut self, name: impl ToString, val: RefVal) {
        let name = name.to_string();
        if let Some(entry) = self.variables.get_mut(&name) {
            entry.push(val);
        } else {
            self.variables.insert(name, vec![val]);
        }
    }

    pub fn unbind_var(&mut self, name: &str) -> Result<(), String> {
        if let Some(entry) = self.variables.get_mut(name) {
            let popped = entry.pop();

            // As soon as the vector is empty, we remove the entry. Therefore it
            // shouldn't be possible to fail this assertion.
            assert!(popped.is_some());

            if entry.len() == 0 {
                self.variables.remove(name);
            }

            Ok(())
        } else {
            Err("variable not bound".to_string())
        }
    }

    pub fn lookup_var(&self, name: &str) -> Option<&RefVal> {
        self.variables.get(name).and_then(|vars| vars.iter().last())
    }
}

pub fn evaluate(expr: &SExpr, env: &mut Environment) -> Result<RefVal, String> {
    match expr {
        SExpr::Atom(atom) => match atom {
            Atom::Ident(ident) => env
                .lookup_var(ident)
                .ok_or(format!("name '{ident}' was not defined"))
                .cloned(),

            Atom::String(s) => Ok(RefVal::owned(Value::String(s.clone()))),
            Atom::Number(n) => Ok(RefVal::owned(Value::Number(*n))),
            Atom::Quote(box q) => Ok(RefVal::owned(Value::Quote(q.clone()))),
        },

        SExpr::List(elements) => {
            let values: Vec<_> = elements
                .into_iter()
                .map(|expr| evaluate(expr, env))
                .collect::<Result<_, _>>()?;

            let fun = values
                .get(0)
                .ok_or("expected list to have at least one element".to_string())?
                .clone();

            if let Value::Function(fun) = fun.borrow() {
                if fun.arity() != values[1..].len() {
                    return Err(format!(
                        "expected {} arguments, but got {} in {:?}",
                        fun.arity(),
                        values[1..].len(),
                        fun
                    ));
                }
                env.stack.extend(values[1..].iter().cloned());
                call(fun, env)
            } else {
                Err(format!("expected a function got `{}`", fun))
            }
        }
    }
}

pub fn call(func: &Function, env: &mut Environment) -> Result<RefVal, String> {
    match func {
        Function::UserDefined { arg_names, body } => {
            let args = env.stack.split_off(env.stack.len() - func.arity());
            for (name, val) in arg_names.iter().zip(args.into_iter()) {
                env.bind_var(name, val);
            }

            let retr = evaluate(body, env)?;

            for name in arg_names.iter() {
                env.unbind_var(name.as_ref())?;
            }

            Ok(retr)
        }

        Function::Lib { ptr, .. } => (*ptr)(env),
    }
}
