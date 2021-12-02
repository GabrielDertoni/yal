use std::rc::Rc;

use crate::evaluator::Environment;

type Boxed<T> = Rc<T>;

#[derive(Debug, Clone)]
pub enum Atom {
    String(Boxed<String>),
    Number(f64),
    Quote(Boxed<SExpr>),
    Ident(Boxed<String>),
    Function(Boxed<Function>),
    Nil,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SExpr {
    Atom(Atom),
    Cons(Atom, Atom),
}

#[derive(Clone)]
pub enum Function {
    UserDefined {
        arg_names: Vec<String>,
        body: Rc<SExpr>,
    },
    Lib {
        name: &'static str,
        ptr: fn(&mut Environment) -> Result<Atom, String>,
        arity: usize,
    },
}

impl Atom {
    pub fn quote(expr: SExpr) -> Atom {
        Atom::Quote(Rc::new(expr))
    }

    pub fn as_quote(&self) -> Option<&SExpr> {
        if let Self::Quote(v) = self {
            Some(v.as_ref())
        } else {
            None
        }
    }

    pub fn as_ident(&self) -> Option<&String> {
        if let Self::Ident(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_function(&self) -> Option<&Function> {
        if let Self::Function(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn try_into_ident(self) -> Result<Rc<String>, Self> {
        if let Self::Ident(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }

    pub fn try_into_quote(self) -> Result<Rc<SExpr>, Self> {
        if let Self::Quote(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }

    pub fn get_type(&self) -> &'static str {
        use Atom::*;

        match self {
            String(_)   => "string",
            Number(_)   => "number",
            Quote(_)    => "quote",
            Ident(_)    => "ident",
            Function(_) => "function",
            Nil         => "nil",
        }
    }
}

impl SExpr {
    pub fn as_atom(&self) -> Option<&Atom> {
        if let Self::Atom(v) = self {
            Some(v)
        } else {
            None
        }
    }
}


impl Function {
    pub fn arity(&self) -> usize {
        use Function::*;

        match self {
            UserDefined { arg_names, .. } => arg_names.len(),
            Lib { arity, .. } => *arity,
        }
    }
}

impl From<Atom> for SExpr {
    fn from(atom: Atom) -> SExpr {
        SExpr::Atom(atom)
    }
}

impl PartialEq for Atom {
    fn eq(&self, rhs: &Atom) -> bool {
        use Atom::*;

        match (self, rhs) {
            (Number(lhs), Number(rhs))     => lhs == rhs,
            (String(lhs), String(rhs))     => *lhs == *rhs,
            (Quote(lhs), Quote(rhs))       => *lhs == *rhs,
            (Ident(lhs), Ident(rhs))       => *lhs == *rhs,
            (Function(lhs), Function(rhs)) => Rc::as_ptr(lhs) == Rc::as_ptr(rhs),
            (Nil, Nil)                     => true,
            _                              => false,
        }
    }
}

use std::fmt::{ self, Debug, Display, Formatter };

impl Display for SExpr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            SExpr::Atom(atom) => Display::fmt(atom, f),
            SExpr::Cons(head, tail) => write!(f, "({} . {})", head, tail),
        }
    }
}

impl Display for Atom {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use Atom::*;

        match self {
            String(s)     => write!(f, "\"{s}\""),
            Number(n)     => Display::fmt(n, f),
            Quote(q)      => write!(f, "'{q}"),
            Ident(i)      => Display::fmt(i, f),
            Function(fun) => Display::fmt(fun, f),
            Nil           => write!(f, "nil"),
        }
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use Function::*;

        match self {
            UserDefined { arg_names, .. } => {
                write!(f, "user function with {} arguments", arg_names.len())
            }

            Lib { name, arity, .. } => {
                write!(f, "lib function '{}' with {} arguments", name, arity)
            }
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
