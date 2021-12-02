use std::ops::Deref;
use std::rc::Rc;

use crate::ast::*;
use crate::error::RuntimeError;
use crate::evaluator::*;

// TODOOO: This should be scoped, somehow
pub fn let_impl(env: &mut Environment) -> Result<Atom, RuntimeError> {
    let val = env.pop_stack();
    let name = env.pop_stack();

    let name = name
        .as_quote()
        .and_then(SExpr::as_atom)
        .and_then(Atom::as_ident)
        .ok_or(format!("expected a symbol, got {:?}", name))?;

    env.bind_var(name, val.clone());
    Ok(val)
}

pub fn fn_impl(env: &mut Environment) -> Result<Atom, RuntimeError> {
    let body = env.pop_stack();
    let args = env.pop_stack();

    let args = args.as_quote();
    let mut args = args.as_deref()
        .ok_or(format!("expected arguments, got {:?}", args))?;

    let mut arg_names = Vec::new();
    while let SExpr::Cons(arg, tail) = args {
        let arg = arg
            .as_ident()
            .ok_or(format!("expected argument, got `{arg}`"))?;

        arg_names.push(arg.clone());
        if let Some(next) = tail.as_quote() {
            args = next;
        } else {
            // FIXME: Is this correct??
            break;
        }
    }

    let body = body
        .try_into_quote()
        .map_err(|body| format!(
            "expected function body to be a list of expressions, got {:?}",
            body
        ))?
        .clone();

    Ok(Atom::Function(Rc::new(Function::UserDefined {
        arg_names,
        body,
    })))
}

pub fn if_impl(env: &mut Environment) -> Result<Atom, RuntimeError> {
    let else_branch = env.pop_stack();
    let then_branch = env.pop_stack();
    let cond = env.pop_stack();

    let then_branch = then_branch.as_quote().ok_or(format!(
        "expected then branch to be quoted, got {:?}",
        then_branch
    ))?;

    let else_branch = else_branch.as_quote().ok_or(format!(
        "expected else branch to be quoted, got {:?}",
        else_branch
    ))?;

    if let Atom::Nil = cond {
        return evaluate(else_branch.deref(), env);
    }
    evaluate(then_branch.deref(), env)
}

pub fn eval_impl(env: &mut Environment) -> Result<Atom, RuntimeError> {
    let expr = env.pop_stack();

    let expr = expr
        .as_quote()
        .ok_or(format!("expected an expression, got {:?}", expr))?;

    evaluate(expr.deref(), env)
}

pub fn cons_impl(env: &mut Environment) -> Result<Atom, RuntimeError> {
    let tail = env.pop_stack();
    let head = env.pop_stack();
    Ok(Atom::Quote(Rc::new(SExpr::Cons(head, tail))))
}

pub fn car_impl(env: &mut Environment) -> Result<Atom, RuntimeError> {
    let list = env.pop_stack();

    if let Some(SExpr::Cons(head, _)) = list.as_quote().as_deref() {
        Ok(head.clone())
    } else {
        return Err(format!("car expected a list, got {}", list));
    }
}

pub fn cdr_impl(env: &mut Environment) -> Result<Atom, RuntimeError> {
    let list = env.pop_stack();

    if let Some(SExpr::Cons(_, tail)) = list.as_quote().as_deref() {
        Ok(tail.clone())
    } else {
        return Err(format!("cdr expected a list, got {}", list));
    }
}

pub fn eq(env: &mut Environment) -> Result<Atom, RuntimeError> {
    use Atom::*;

    let rhs = env.pop_stack();
    let lhs = env.pop_stack();

    let res = match (lhs, rhs) {
        (String(lhs), String(rhs)) if lhs == rhs => true,
        (Number(lhs), Number(rhs)) if lhs == rhs => true,
        (Quote(lhs), Quote(rhs)) if *lhs == *rhs => true,
        (Function(lhs), Function(rhs)) if Rc::as_ptr(&lhs) == Rc::as_ptr(&rhs) => true,
        _ => false,
    };

    // FIXME: This might be a bug if the user overwrites the variable "t".
    Ok(if res { env.lookup_var("t").unwrap() } else { Atom::Nil })
}

macro_rules! impl_bin_op {
    () => {};

    (@once pub fn $name:ident => $op:tt) => {
        #[allow(dead_code)]
        pub fn $name(env: &mut Environment) -> Result<Atom, RuntimeError> {
            use Atom::Number;

            let rhs = env.pop_stack();
            let lhs = env.pop_stack();

            match (&lhs, &rhs) {
                (Number(lhs), Number(rhs)) => Ok(Number(lhs $op rhs)),
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

pub fn print_impl(env: &mut Environment) -> Result<Atom, RuntimeError> {
    print!("{}", env.pop_stack());
    Ok(Atom::Nil)
}

pub fn dbg_impl(env: &mut Environment) -> Result<Atom, RuntimeError> {
    eprintln!("{:#?}", env.pop_stack());
    Ok(Atom::Nil)
}
