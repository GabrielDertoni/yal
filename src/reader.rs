use std::str::pattern::Pattern;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::ast::*;
use crate::error::*;

pub struct Reader<'a> {
    source: &'a str,
    chars: ParenChars<'a>,
}

impl<'a> Reader<'a> {
    const IDENT_CHARS: &'static str = "_+-/*=?";

    pub fn new(source: &'a str) -> Reader<'a> {
        Reader {
            source,
            chars: ParenChars::new(source),
        }
    }

    fn advance(&mut self) -> Option<char> {
        self.chars.next()
    }

    fn peek(&self) -> Option<char> {
        self.chars.peek()
    }

    fn rest(&self) -> &'a str {
        self.chars.as_str()
    }

    fn idx(&self) -> usize {
        self.source[..self.source.len() - self.rest().len()].chars().count()
    }

    fn pos(&self) -> Position<'a> {
        Position {
            src: self.source,
            byte: self.source.len() - self.rest().len()
        }
    }

    fn error(&self, msg: impl ToString) -> Error<'a> {
        Error::new(self.source, self.idx(), msg)
    }

    fn skip_whitespace(&mut self) {
        while let Some(chr) = self.peek() {
            if !chr.is_whitespace() { return }
            self.advance();
        }
    }

    pub fn parse_atom(&mut self) -> Result<Atom, Error<'a>> {
        match self.peek().unwrap() {
            '"' => {
                self.advance();
                let start = self.pos();
                while let Some(chr) = self.peek() {
                    if chr == '\\' {
                        self.advance();
                    }
                    else if chr == '"' { break }
                    self.advance();
                }
                let s = start.span_to(self.pos()).as_str().to_string();

                // TODOO: Make this more efficient!
                let s = s.replace("\\n", "\n");
                let s = s.replace("\\r", "\0");
                let s = s.replace("\\0", "\0");
                self.advance();
                Ok(Atom::String(Rc::new(s)))
            }

            '\'' => {
                self.advance();
                Ok(Atom::Quote(Rc::new(self.parse_sexpr()?)))
            },

            chr if chr.is_digit(10) => {
                let mut read_dot = false;
                let start = self.pos();
                while let Some(chr) = self.peek() {
                    if chr == '.' && !read_dot {
                        read_dot = true;
                    }
                    if !chr.is_digit(10) || (chr == '.' && read_dot) { break }
                    self.advance();
                }

                let tok = start.span_to(self.pos()).as_str().to_string();
                let num = tok
                    .parse()
                    .map_err(|_| self.error(format!("number in wrong format '{tok}'")))?;

                Ok(Atom::Number(num))
            }

            chr if chr.is_whitespace() => Err(self.error("unexpected whitespace")),

            chr if chr.is_alphabetic() || chr.is_contained_in(Self::IDENT_CHARS) => {
                let start = self.pos();
                while let Some(chr) = self.peek() {
                    if !(chr.is_alphanumeric() || chr.is_contained_in(Self::IDENT_CHARS)) {
                        break
                    }
                    self.advance();
                }

                Ok(Atom::Ident(Rc::new(start.span_to(self.pos()).as_str().to_string())))
            }

            chr => Err(self.error(format!("unexpected char '{chr}'"))),
        }
    }

    pub fn parse_sexpr(&mut self) -> Result<SExpr, Error<'a>> {
        loop {
            match self.peek() {
                Some('(') => {
                    self.advance();
                    let mut sub_reader = Reader {
                        source: self.source,
                        chars: ParenChars::new(self.rest()),
                    };
                    let sexprs = sub_reader.parse_sexprs()?;
                    self.chars.merge(sub_reader.chars);
                    if self.peek() != Some(')') {
                        return Err(self.error("expected a closing paren"));
                    }
                    self.advance();

                    let mut list = SExpr::Atom(Atom::Nil);
                    for sexpr in sexprs.into_iter().rev() {
                        list = SExpr::Cons(Atom::quote(sexpr), Atom::quote(list));
                    }
                    return Ok(list)
                },

                Some(';') => {
                    while let Some(chr) = self.peek() {
                        if chr == '\n' { break }
                        self.advance();
                    }
                    self.skip_whitespace();
                }

                Some(_) => return Ok(SExpr::Atom(self.parse_atom()?)),
                None => return Err(self.error("unexpected end of input")),
            }
        }
    }

    pub fn parse_sexprs(&mut self) -> Result<VecDeque<SExpr>, Error<'a>> {
        let mut s_exprs = VecDeque::new();

        loop {
            self.skip_whitespace();
            if self.peek().is_none() {
                return Ok(s_exprs)
            }
            s_exprs.push_back(self.parse_sexpr()?);
        }
    }
}

pub struct ParenChars<'a> {
    slice: &'a str,
    next: Option<char>,
    level: i32,
    // TODO: Make this go away!
    in_str: bool,
    in_escape: bool,
}

impl<'a> ParenChars<'a> {
    pub fn new(s: &'a str) -> Self {
        Self::with_level(s, 0)
    }

    pub fn with_level(s: &'a str, level: i32) -> Self {
        let next = s.chars().next();
        ParenChars {
            slice: s,
            next,
            level,
            in_str: false,
            in_escape: false,
        }
    }

    pub fn as_str(&self) -> &'a str {
        self.slice
    }

    pub fn peek(&self) -> Option<char> {
        if self.next == Some(')') && self.level == 0 && !self.in_str {
            None
        } else {
            self.next
        }
    }

    pub fn merge(&mut self, other: ParenChars<'a>) {
        unsafe {
            assert!(
                self.slice.as_ptr() <= other.slice.as_ptr(),
                "Slices must overlap"
            );
            assert!(
                self.slice.as_ptr().add(self.slice.len()) >= other.slice.as_ptr(),
                "Slices must overlap"
            );
            assert!(
                self.slice.as_ptr().add(self.slice.len()) == other.slice.as_ptr().add(other.slice.len()),
                "Slices must end in the same place"
            );
        }
        self.level += other.level;
        self.slice = other.slice;
        self.next = other.next;
    }
}

impl<'a> Iterator for ParenChars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        let prev = self.next?;
        self.slice = &self.slice[prev.len_utf8()..];

        // Let's hope that this `.nth(0)` is not terribly inefficient.
        self.next = self.slice.chars().nth(0);

        let was_escape = self.in_escape;

        if prev == '"' && !self.in_escape {
            self.in_str = !self.in_str;
        } else if prev == '\\' {
            self.in_escape = true;
        }

        if was_escape {
            self.in_escape = false;
        }

        if self.in_str {
            return Some(prev);
        }

        if prev == ')' {
            self.level -= 1;
            if self.level < 0 {
                return None;
            }
        } else if prev == '(' {
            self.level += 1;
        }
        Some(prev)
    }
}

#[derive(Clone, Copy)]
pub struct Span<'a> {
    src: &'a str,
    start: usize,
    end: usize
}

impl<'a> Span<'a> {
    fn as_str(&self) -> &'a str {
        &self.src[self.start..self.end]
    }
}

#[derive(Clone, Copy)]
pub struct Position<'a> {
    src: &'a str,
    byte: usize,
}

impl<'a> Position<'a> {
    fn span_to(&self,end: Position<'a>) -> Span<'a> {
        Span {
            src: self.src,
            start: self.byte,
            end: end.byte,
        }
    }
}

