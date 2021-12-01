use std::fmt::{ Display, Debug, Formatter, Result };

pub type RuntimeError = String;

pub struct Error<'a> {
    src: &'a str,
    char_idx: usize,
    msg: String,
}

impl<'a> Error<'a> {
    pub fn new(src: &'a str, byte: usize, msg: impl ToString) -> Self {
        Error {
            src,
            char_idx: byte,
            msg: msg.to_string(),
        }
    }
}

impl<'a> Display for Error<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut line = 1;
        let mut col = 1;
        for (i, chr) in self.src.char_indices() {
            if i > self.char_idx { break }
            if chr == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        write!(f, "{} at {}:{}", self.msg, line, col)
    }
}

impl<'a> Debug for Error<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        Display::fmt(self, f)
    }
}

impl<'a> std::error::Error for Error<'a> {
    fn description(&self) -> &str {
        self.msg.as_str()
    }
}
