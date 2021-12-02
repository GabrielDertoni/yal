use std::ops::Deref;

use lazy_static::lazy_static;

use crate::ast::*;
use crate::error::RuntimeError;
use crate::evaluator::*;

lazy_static! {
    static ref TRUE: Value = Value::Quote(SExpr::Atom(Atom::Ident("true".to_string())));
    static ref FALSE: Value = Value::Quote(SExpr::Atom(Atom::Ident("false".to_string())));
    static ref NIL: Value = Value::Quote(SExpr::Atom(Atom::Ident("nil".to_string())));
}

fn true_ref() -> &'static Value {
    TRUE.deref()
}

fn false_ref() -> &'static Value {
    FALSE.deref()
}

fn nil_ref() -> &'static Value {
    NIL.deref()
}

fn symbol(s: impl ToString) -> RefVal {
    RefVal::owned(Value::Quote(SExpr::Atom(Atom::Ident(s.to_string()))))
}

impl Into<RefVal> for bool {
    fn into(self) -> RefVal {
        match self {
            true => RefVal::reference(true_ref()),
            false => RefVal::reference(false_ref()),
        }
    }
}

impl Into<RefVal> for String {
    fn into(self) -> RefVal {
        RefVal::owned(Value::String(self))
    }
}

impl Into<RefVal> for f64 {
    fn into(self) -> RefVal {
        RefVal::owned(Value::Number(self))
    }
}

impl From<SExpr> for Atom {
    fn from(expr: SExpr) -> Atom {
        Atom::Quote(Box::new(expr))
    }
}

impl From<Atom> for SExpr {
    fn from(atom: Atom) -> SExpr {
        SExpr::Atom(atom)
    }
}

pub fn let_impl(env: &mut Environment) -> Result<RefVal, RuntimeError> {
    let val = env.pop_stack();
    let name = env.pop_stack();

    let name = name
        .deref()
        .as_quote()
        .and_then(SExpr::as_atom)
        .and_then(Atom::as_ident)
        .ok_or(format!("expected a symbol, got {:?}", name))?;

    env.register_var(name, val.clone());
    Ok(val)
}

pub fn fn_impl(env: &mut Environment) -> Result<RefVal, RuntimeError> {
    let body = env.pop_stack();
    let args = env.pop_stack();

    let args = args
        .deref()
        .as_quote()
        .and_then(SExpr::as_list)
        .ok_or(format!("expected arguments, got {:?}", args))?;

    let mut arg_names = Vec::new();
    for arg in args {
        let arg = arg
            .as_atom()
            .and_then(Atom::as_ident)
            .ok_or(format!("expected argument, got {:?}", arg))?;

        arg_names.push(arg.clone())
    }

    let body = body
        .deref()
        .as_quote()
        .ok_or(format!(
            "expected function body to be a list of expressions, got {:?}",
            body
        ))?
        .clone();

    Ok(RefVal::owned(Value::Function(Function::UserDefined {
        arg_names,
        body,
    })))
}

pub fn if_impl(env: &mut Environment) -> Result<RefVal, RuntimeError> {
    let else_branch = env.pop_stack();
    let then_branch = env.pop_stack();
    let cond = env.pop_stack();

    let then_branch = then_branch.deref().as_quote().ok_or(format!(
        "expected then branch to be quoted, got {:?}",
        then_branch
    ))?;

    let else_branch = else_branch.deref().as_quote().ok_or(format!(
        "expected else branch to be quoted, got {:?}",
        else_branch
    ))?;

    if let RefVal::Borrowed(b) = cond {
        let ptr = b as *const Value;
        if ptr == false_ref() as *const Value || ptr == nil_ref() as *const Value {
            return evaluate(else_branch, env);
        }
    }
    evaluate(then_branch, env)
}

pub fn eval_impl(env: &mut Environment) -> Result<RefVal, RuntimeError> {
    let expr = env.pop_stack();

    let expr = expr
        .deref()
        .as_quote()
        .ok_or(format!("expected an expression, got {:?}", expr))?;

    evaluate(expr, env)
}

pub fn cons_impl(env: &mut Environment) -> Result<RefVal, RuntimeError> {
    let tail = env.pop_stack();
    let head = env.pop_stack();

    let head = head
        .deref()
        .as_quote()
        .ok_or(format!("expected a quoted expression, got {:?}", head))?;

    let mut tail = tail
        .deref()
        .as_quote()
        .and_then(SExpr::as_list)
        .ok_or(format!("expected a quoted expression, got {:?}", tail))?
        .clone();

    tail.push_front(head.clone());
    Ok(RefVal::owned(Value::Quote(SExpr::List(tail))))
}

pub fn car_impl(env: &mut Environment) -> Result<RefVal, RuntimeError> {
    let list = env.pop_stack();

    let list = list
        .deref()
        .as_quote()
        .and_then(SExpr::as_list)
        .ok_or(format!("expected a list, got {:?}", list))?;

    Ok(RefVal::owned(Value::Quote(
        list.get(0)
            .ok_or(format!("expected non empty list"))?
            .clone(),
    )))
}

pub fn cdr_impl(env: &mut Environment) -> Result<RefVal, RuntimeError> {
    let list = env.pop_stack();

    let list = list
        .deref()
        .as_quote()
        .and_then(SExpr::as_list)
        .ok_or(format!("expected a list, got {:?}", list))?;

    if list.len() == 0 {
        return Err(format!("expected non empty list"));
    }

    Ok(RefVal::owned(Value::Quote(SExpr::List(
        list.iter()
            .skip(1)
            .cloned()
            .collect(),
    ))))
}

pub fn eq(env: &mut Environment) -> Result<RefVal, RuntimeError> {
    use Value::*;

    let rhs = env.pop_stack();
    let lhs = env.pop_stack();

    Ok(match (lhs.deref(), rhs.deref()) {
        (String(lhs), String(rhs)) if lhs == rhs => true,
        (Number(lhs), Number(rhs)) if lhs == rhs => true,
        (Quote(lhs), Quote(rhs)) if lhs == rhs => true,
        (Function(_), Function(_)) if &lhs.as_ptr() == &rhs.as_ptr() => true,
        _ => false,
    }
    .into())
}

macro_rules! impl_bin_op {
    () => {};

    (@once pub fn $name:ident => $op:tt) => {
        #[allow(dead_code)]
        pub fn $name(env: &mut Environment) -> Result<RefVal, RuntimeError> {
            use Value::*;

            let rhs = env.pop_stack();
            let lhs = env.pop_stack();

            match (lhs.deref(), rhs.deref()) {
                (Number(lhs), Number(rhs)) => Ok((lhs $op rhs).into()),
                _ => {
                    Err(format!(
                        "expected two numbers in operation '{}', got {} and {}",
                        stringify!($op),
                        lhs.get_type(),
                        rhs.get_type()
                    ))
                }
            }
        }
    };

    (pub fn $name:ident => $op:tt; $($tail:tt)*) => {
        impl_bin_op! { @once pub fn $name => $op }
        impl_bin_op! { $($tail)* }
    };
}

impl_bin_op! {
    pub fn sub => -;
    pub fn add => +;
    pub fn mul => *;
    pub fn div => /;
}

pub fn print_impl(env: &mut Environment) -> Result<RefVal, RuntimeError> {
    println!("{}", env.pop_stack());
    Ok(RefVal::reference(nil_ref()))
}
