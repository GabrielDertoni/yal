#![feature(pattern)]
#![feature(box_patterns)]
#![feature(result_cloned)]

mod error;
mod ast;
mod reader;
mod evaluator;
mod std_lib;

use std::{ fs, env };

use reader::Reader;
use evaluator::*;

/*
macro_rules! try_res {
    ($($tok:tt)*) => {
        (|| -> Result<_, _> { $($tok)* })()
    };
}

macro_rules! try_opt {
    ($($tok:tt)*) => {
        (|| -> Option<_> { $($tok)* })()
    };
}
*/


fn main() -> Result<(), Box<dyn std::error::Error>>{
    let mut args = env::args();

    // Ignore the program name.
    args.next();

    let fname = args.next().ok_or("Expected a file name")?;

    let contents = fs::read_to_string(fname)?;
    let mut reader = Reader::new(&contents);
    let s_exprs = match reader.parse_sexprs() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            return Ok(());
        },
    };

    let mut env = Environment::new();

    env.register_external_fun("let", 2, std_lib::let_impl);
    env.register_external_fun("fn", 2, std_lib::fn_impl);
    env.register_external_fun("if", 3, std_lib::if_impl);
    env.register_external_fun("eval", 1, std_lib::eval_impl);
    env.register_external_fun("cons", 2, std_lib::cons_impl);
    env.register_external_fun("car", 1, std_lib::car_impl);
    env.register_external_fun("cdr", 1, std_lib::cdr_impl);
    env.register_external_fun("=", 2, std_lib::eq);
    env.register_external_fun("eq", 2, std_lib::eq);
    env.register_external_fun("+", 2, std_lib::add);
    env.register_external_fun("-", 2, std_lib::sub);
    env.register_external_fun("*", 2, std_lib::mul);
    env.register_external_fun("/", 2, std_lib::div);
    env.register_external_fun("print", 1, std_lib::print_impl);

    for expr in s_exprs {
        evaluate(&expr, &mut env)?;
    }

    Ok(())
}
