use std::rc::Rc;
use std::borrow::{ ToOwned, Borrow };
use std::ops::Deref;

use crate::evaluator::Environment;

#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    String(String),
    Number(f64),
    Quote(Box<SExpr>),
    Ident(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SExpr {
    List(Vec<SExpr>),
    Atom(Atom),
}

#[derive(Debug)]
pub enum Value {
    String(String),
    Number(f64),
    Quote(SExpr),
    Function(Function),
}

#[derive(Clone)]
pub enum Function {
    UserDefined {
        arg_names: Vec<String>,
        body: SExpr,
    },
    Lib {
        name: &'static str,
        ptr: fn(&mut Environment) -> Result<RefVal, String>,
        arity: usize,
    },
}

#[derive(Debug, Clone)]
pub struct BoxedVal(Rc<Value>);

#[derive(Debug, Clone)]
pub enum RefVal {
    Borrowed(&'static Value),
    Owned(BoxedVal),
}

impl Atom {
    pub fn as_quote(&self) -> Option<&Box<SExpr>> {
        if let Self::Quote(v) = self {
            Some(v)
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

    pub fn try_into_ident(self) -> Result<String, Self> {
        if let Self::Ident(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }
}

impl SExpr {
    pub fn as_list(&self) -> Option<&Vec<SExpr>> {
        if let Self::List(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_atom(&self) -> Option<&Atom> {
        if let Self::Atom(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl Value {
    pub fn as_quote(&self) -> Option<&SExpr> {
        if let Self::Quote(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl ToOwned for Value {
    type Owned = BoxedVal;

    fn to_owned(&self) -> BoxedVal {
        use Value::*;

        match self {
            String(s) => BoxedVal::new(String(s.clone())),
            Number(n) => BoxedVal::new(Number(n.clone())),
            Quote(q)  => BoxedVal::new(Quote(q.clone())),
            Function(f) => BoxedVal::new(Function(f.clone())),
        }
    }
}

impl Borrow<Value> for BoxedVal {
    fn borrow(&self) -> &Value {
        self.0.as_ref()
    }
}

impl Borrow<Value> for RefVal {
    fn borrow(&self) -> &Value {
        match self {
            RefVal::Borrowed(v) => v,
            RefVal::Owned(o) => o.borrow(),
        }
    }
}

impl BoxedVal {
    pub fn new(val: Value) -> BoxedVal {
        BoxedVal(Rc::new(val))
    }
}

impl RefVal {
    pub fn owned(val: Value) -> RefVal {
        RefVal::Owned(BoxedVal::new(val))
    }

    pub fn reference(reference: &'static Value) -> RefVal {
        RefVal::Borrowed(reference)
    }

    pub fn as_ptr(&self) -> *const Value {
        match self {
            RefVal::Borrowed(b) => *b as *const Value,
            RefVal::Owned(o) => Rc::as_ptr(&o.0),
        }
    }
}

impl Deref for RefVal {
    type Target = Value;

    fn deref(&self) -> &Value {
        match self {
            RefVal::Borrowed(b) => b,
            RefVal::Owned(o)    => o.borrow(),
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

impl std::fmt::Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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
