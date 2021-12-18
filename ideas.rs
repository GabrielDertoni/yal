use std::collections::HashMap;

pub enum Atom {
    String(String),
    Number(f64),
    Symbol(String),
}

pub enum Expr {
    Quote(Rc<Expr>),
    Cons(Rc<Expr>, Rc<Expr>),
    Lambda {
        arg_names: Vec<String>,
        body: Rc<Expr>,
    },
    NativeFn(fn(Vec<Expr>) -> Result<Expr, String>),
    Nil,
}

trait Evaluator {
    fn bind_var(&mut self, name: &str, val: Expr);
    fn evaluate_args(&mut self, expr: &Expr) -> Result<Vec<Expr>, String>;
    fn evaluate(&mut self, expr: &Expr) -> Result<Expr, String>;
}

struct Environment {
    variables: HashMap<String, Expr>,
}

impl Evaluator for Environment {
    fn evaluate_args(&mut self, mut expr: &Expr) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        while let Expr::Cons(head, tail) = expr {
            args.push(self.evaluate(head)?);
            expr = tail.as_ref();
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Expr, String> {
        match expr {
            Expr::Nil => Expr::Nil,
            Expr::Quote(expr) => expr,
            Expr::Cons(fun, args) => {
                let fun = evaluate(fun)?;
                let args = evaluate_args(snd)?;
                match fun {
                    Expr::NativeFn(fun) => fun(args),
                    Expr::Lambda { arg_names, body } => {
                        for (name, val) in fun.arg_names.iter().zip(&args) {
                            self.bind_var(name, val);
                        }
                        evaluate(fun.body)
                    }
                }
            }
        }
    }
}

