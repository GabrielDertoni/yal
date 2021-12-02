use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::*;

#[derive(Debug)]
pub struct Environment {
    scopes: Vec<HashMap<String, Atom>>,
    stack: Vec<Atom>,
}

impl Environment {
    pub fn new() -> Self {
        let mut env = Environment {
            scopes: vec![HashMap::new()],
            stack: Vec::new(),
        };

        let the_true = Atom::Quote(Rc::new(SExpr::Atom(Atom::Ident(Rc::new(String::from(
            "t",
        ))))));

        env.bind_var("t", the_true);
        env
    }

    pub fn pop_stack(&mut self) -> Atom {
        self.stack.pop().unwrap()
    }

    pub fn push_stack(&mut self, val: Atom) {
        self.stack.push(val);
    }

    pub fn scope(&self) -> &HashMap<String, Atom> {
        &self.scopes[self.scopes.len() - 1]
    }

    pub fn scope_mut(&mut self) -> &mut HashMap<String, Atom> {
        let idx = self.scopes.len() - 1;
        &mut self.scopes[idx]
    }

    pub fn push_frame(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_frame(&mut self) {
        self.scopes.pop();
    }

    pub fn register_external_fun(
        &mut self,
        name: &'static str,
        arity: usize,
        ptr: fn(&mut Environment) -> Result<Atom, String>,
    ) {
        self.scope_mut().insert(
            name.to_string(),
            Atom::Function(Rc::new(Function::Lib { name, arity, ptr })),
        );
    }

    pub fn bind_var(&mut self, name: impl ToString, val: Atom) {
        self.scope_mut().insert(name.to_string(), val);
    }

    pub fn unbind_var(&mut self, name: impl AsRef<str>) -> Result<(), String> {
        if self.scope_mut().remove(name.as_ref()).is_some() {
            Ok(())
        } else {
            Err(format!("variable {} not bound", name.as_ref()))
        }
    }

    pub fn lookup_var(&self, name: impl AsRef<str>) -> Option<Atom> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name.as_ref()) {
                return Some(val.clone());
            }
        }
        None
    }
}

pub fn evaluate(expr: &SExpr, env: &mut Environment) -> Result<Atom, String> {
    match expr {
        SExpr::Atom(atom) => match atom {
            Atom::Ident(ident) => env
                .lookup_var(ident.as_ref())
                .ok_or(format!("name '{ident}' was not defined"))
                .clone(),

            atom => Ok(atom.clone()),
        },

        SExpr::Cons(fun, tail) => {
            let mut argc = 0;
            let mut quote = tail.as_quote();
            while let Some(SExpr::Cons(head, next)) = quote {
                let head = head.as_quote().unwrap();
                let res = evaluate(&head.clone().into(), env)?;
                env.push_stack(res);
                quote = next.as_quote();
                argc += 1;
            }

            let fun = fun.as_quote().unwrap();
            let fun = evaluate(fun, env)?;
            let fun = fun
                .as_function()
                .ok_or(format!("expected a function, got `{}`", fun))?;

            if fun.arity() != argc {
                return Err(format!(
                    "expected {} arguments, but got {} in {}",
                    fun.arity(),
                    argc,
                    fun
                ));
            }
            call(fun, env)
        }
    }
}

pub fn call(func: &Function, env: &mut Environment) -> Result<Atom, String> {
    match func {
        Function::UserDefined { arg_names, body } => {
            let args = env.stack.split_off(env.stack.len() - func.arity());
            for (name, val) in arg_names.iter().zip(args.into_iter()) {
                env.bind_var(name, val);
            }

            env.push_frame();
            let retr = evaluate(body, env)?;
            env.pop_frame();

            Ok(retr)
        }

        Function::Lib { ptr, .. } => (*ptr)(env),
    }
}
