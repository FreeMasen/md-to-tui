use std::{fmt::Display, u8};

use crate::error::Error;

const INDENT_CHARS: &[u8; 65] =
    b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890,\"\'";

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub enum Token {
    Heading(usize),
    Indent(String),

    WhiteSpace,
    Tab,

    EOL,
    #[default]
    EOF,

    LeftSquare,
    RightSquare,
    LeftParen,
    RightParen,
    LeftAngle,
    RightAngle,

    Dot,
    Dash,
    Equal,
    Plus,
    Asterisk,
    Undersocre,
    BackTick,
    BackSlash,
    Slash,
    Colon,
    SemiColon,

    Illegal(u8),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Token -> ")?;
        let simple: &str = match self {
            Token::Heading(i) => return write!(f, "Heading: #{}", i),
            Token::Indent(s) => return write!(f, "Indent: {} ", s),
            Token::Illegal(s) => return write!(f, "Illegal: {} ", s),

            Token::WhiteSpace => "WhiteSpace",
            Token::Tab => "Tab",
            Token::EOL => "EOL",
            Token::EOF => "EOF",

            Token::LeftSquare => "LeftSquare",
            Token::RightSquare => "RightSquare",
            Token::LeftParen => "LeftParen",
            Token::RightParen => "RightParen",
            Token::LeftAngle => "LeftAngle",
            Token::RightAngle => "RightAngle",

            Token::Dot => "Dot",
            Token::Dash => "Dash",
            Token::Equal => "Equal",
            Token::Plus => "Plus",
            Token::Asterisk => "Asterisk",
            Token::Undersocre => "Undersocre",
            Token::BackTick => "BackTick",
            Token::BackSlash => "BackSlash",
            Token::Colon => "Colon",
            Token::SemiColon => "SemiColon",
            Token::Slash => "Slash",
        };
        write!(f, "{simple}")
    }
}

impl Token {
    pub fn is_end(&self) -> bool {
        if (*self == Token::EOF) | (*self == Token::EOL) {
            return true;
        }
        false
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Lexer {
    position: usize,
    read_position: usize,
    ch: u8,
    input: Vec<u8>,
}

#[allow(dead_code)]
impl Lexer {
    pub fn new() -> Lexer {
        return Lexer {
            position: 0,
            read_position: 0,
            ch: 0,
            input: "".into(),
        };
    }

    pub fn parse<T: ToString>(&mut self, input: &T) -> Result<Vec<Token>, Error> {
        // BUG: format!("\n{}") is needed becuze it skips first line
        self.input = format!("\n{}", input.to_string()).into();
        // self.input = input.to_string().into();

        let mut tokens: Vec<Token> = Vec::new();
        self.next_token()?;
        while !(self.position >= self.input.len()) {
            tokens.push(self.next_token()?);
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, Error> {
        let tk = match self.ch {
            b' ' => Token::WhiteSpace,
            b'[' => Token::LeftSquare,
            b']' => Token::RightSquare,
            b')' => Token::LeftParen,
            b'(' => Token::RightParen,
            b'<' => Token::LeftAngle,
            b'>' => Token::RightAngle,
            b'-' => Token::Dash,
            b'+' => Token::Plus,
            b'=' => Token::Equal,
            b'#' => return Ok(self.read_heading()),
            ch if INDENT_CHARS.contains(&ch) => return Ok(self.read_indent()),
            b'\0' => Token::EOF,
            b'\n' => Token::EOL,

            b'.' => Token::Dot,
            b'_' => Token::Undersocre,
            b'`' => Token::BackTick,
            b'\\' => Token::BackSlash,
            b'*' => Token::Asterisk,
            b':' => Token::Colon,
            b';' => Token::SemiColon,
            b'/' => Token::Slash,
            b'\t' => Token::Tab,
            _ => Token::Illegal(self.ch),
        };

        match tk {
            Token::Illegal(_) => return Err(Error::LexerErr(tk.to_string())),
            _ => (),
        }

        self.read_char();
        Ok(tk)
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = b'\0';
        } else {
            self.ch = self.input[self.read_position]
        }
        self.position = self.read_position;
        self.read_position += 1;
    }

    fn peek(&mut self) -> u8 {
        if self.read_position >= self.input.len() {
            return 0;
        } else {
            self.input[self.read_position]
        }
    }

    fn read_indent(&mut self) -> Token {
        let pos = self.position;
        while INDENT_CHARS.contains(&self.ch) {
            self.read_char()
        }
        return Token::Indent(String::from_utf8_lossy(&self.input[pos..self.position]).to_string());
    }

    fn read_heading(&mut self) -> Token {
        let pos = self.position;
        while self.ch == b'#' {
            self.read_char()
        }
        Token::Heading(self.position - pos)
    }
}

#[cfg(test)]
mod test {
    use anyhow::{Ok, Result};

    use super::{Lexer, Token};

    #[test]
    fn get_next_token() -> Result<()> {
        let input = r"# Test + --243a,.p ## test lol
2
";

        let tokens = vec![
            Token::EOL,
            Token::Heading(1),
            Token::WhiteSpace,
            Token::Indent("Test".into()),
            Token::WhiteSpace,
            Token::Plus,
            Token::WhiteSpace,
            Token::Dash,
            Token::Dash,
            Token::Indent("243a,".into()),
            Token::Dot,
            Token::Indent("p".into()),
            Token::WhiteSpace,
            Token::Heading(2),
            Token::WhiteSpace,
            Token::Indent("test".into()),
            Token::WhiteSpace,
            Token::Indent("lol".into()),
            Token::EOL,
            Token::Indent("2".into()),
            Token::EOL,
        ];

        let mut lexer = Lexer::new();

        let res = lexer.parse::<&str>(&input)?;

        let mut diff: Vec<(Token, Token)> = vec![];
        for (i, t) in tokens.iter().enumerate() {
            if *t != res[i] {
                diff.push((t.clone(), res[i].clone()))
            }
        }

        println!("DIFF {:?}", diff);
        println!("TOKENS: {:?}\nres {:?}\n", tokens, res);
        assert_eq!(tokens, res);

        Ok(())
    }

    #[test]
    fn dummy() {
        let text = r"
    ### TODO

    use to do that in the 
lol

- lol 
";

        let mut lexer = Lexer::new();

        let res = lexer.parse::<&str>(&text);

        print!("{:?}", res);

        assert_eq!(true, true);
    }
}
