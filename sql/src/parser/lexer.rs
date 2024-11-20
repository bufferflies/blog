use std::fmt::Display;

use crate::error::{Error, Result};

pub struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// A numeric string, with digits, decimal points, and/or exponents. Leading
    /// signs (e.g. -) are separate tokens.
    Number(String),
    /// A Unicode string, with quotes stripped and escape sequences resolved.
    String(String),
    /// An identifier, with any quotes stripped.
    Ident(String),
    /// A SQL keyword.
    Keyword(Keyword),

    Equal,              // =
    NotEqual,           // !=
    LessThan,           // <
    LessThanOrEqual,    // <=
    GreaterThan,        // >
    GreaterThanOrEqual, // >=

    Plus,        // +
    Minus,       // -
    Asterisk,    // *
    Slash,       // /
    Percent,     // %
    Exponential, // ^

    Comma,       // ,
    CloseParen,  // )
    Exclamation, // !
    OpenParen,   // (
    Period,      // .
    Semicolon,   // ;
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            Self::Number(n) => n,
            Self::String(s) => s,
            Self::Ident(s) => s,
            Self::Keyword(k) => return k.fmt(f),
            Self::Period => ".",

            Self::Equal => "=",
            Self::NotEqual => "!=",
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",

            Self::Plus => "+",
            Self::Minus => "-",
            Self::Asterisk => "*",
            Self::Slash => "/",
            Self::Percent => "%",
            Self::Exponential => "^",

            Self::Comma => ",",
            Self::Semicolon => ";",
            Self::OpenParen => "(",
            Self::CloseParen => ")",
            Self::Exclamation => "!",
        })
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.scan() {
            Ok(Some(token)) => Some(Ok(token)),
            Ok(None) => self
                .chars
                .peek()
                .map(|c| Err(Error::InvalidInput(format!("Unexpected character:{}", c)))),
            Err(e) => Some(Err(e)),
        }
    }
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }

    fn scan(&mut self) -> Result<Option<Token>> {
        self.skip_whitespace();
        match self.chars.peek() {
            Some('\'') => self.scan_string(),
            Some('"') => self.scan_ident_quoted(),
            Some(c) if c.is_ascii_digit() => Ok(self.scan_number()),
            Some(c) if c.is_ascii_alphabetic() => Ok(self.scan_ident_or_keyword()),
            Some(_) => Ok(self.scan_symbol()),
            None => Ok(None),
        }
    }

    fn scan_symbol(&mut self) -> Option<Token> {
        let mut token = self.next_if_map(|c| {
            Some(match c {
                '=' => Token::Equal,
                '!' => Token::Exclamation,
                '<' => Token::LessThan,
                '>' => Token::GreaterThan,

                '+' => Token::Plus,
                '-' => Token::Minus,
                '*' => Token::Asterisk,
                '/' => Token::Slash,
                '%' => Token::Percent,
                '^' => Token::Exponential,

                ';' => Token::Semicolon,
                '.' => Token::Period,
                ',' => Token::Comma,
                '(' => Token::OpenParen,
                ')' => Token::CloseParen,

                _ => return None,
            })
        })?;
        token = match token {
            Token::Exclamation if self.next_is('=') => Token::NotEqual,
            Token::LessThan if self.next_is('=') => Token::LessThanOrEqual,
            Token::GreaterThan if self.next_is('=') => Token::GreaterThanOrEqual,
            token => token,
        };
        Some(token)
    }

    fn scan_string(&mut self) -> Result<Option<Token>> {
        if !self.next_is('\'') {
            return Ok(None);
        }

        let mut string = String::new();
        loop {
            match self.chars.next() {
                Some('\'') => break,
                Some(c) => string.push(c),
                None => return Error::InvalidInput("Unterminated string".to_owned()).into(),
            }
        }

        Ok(Some(Token::String(string)))
    }

    fn scan_ident_quoted(&mut self) -> Result<Option<Token>> {
        if !self.next_is('"') {
            return Ok(None);
        }

        let mut ident = String::new();
        loop {
            match self.chars.next() {
                Some('"') => break,
                Some(c) => ident.push(c),
                None => {
                    return Error::InvalidInput("Unterminated quoted identifier".to_owned()).into();
                }
            }
        }

        Ok(Some(Token::Ident(ident)))
    }

    fn scan_number(&mut self) -> Option<Token> {
        let mut number = self.next_if(|c| c.is_ascii_digit())?.to_string();
        while let Some(c) = self.next_if(|c| c.is_ascii_digit()) {
            number.push(c);
        }
        if self.next_is('.') {
            number.push('.');
            while let Some(c) = self.next_if(|c| c.is_ascii_digit()) {
                number.push(c);
            }
        }

        if let Some(exp) = self.next_if(|c| c == 'e' || c == 'E') {
            number.push(exp);
            if let Some(sign) = self.next_if(|c| c == '-' || c == '+') {
                number.push(sign);
            }
            while let Some(c) = self.next_if(|c| c.is_ascii_digit()) {
                number.push(c);
            }
        }
        Some(Token::Number(number))
    }

    fn scan_ident_or_keyword(&mut self) -> Option<Token> {
        let mut name = self
            .next_if(|c| c.is_alphabetic())?
            .to_lowercase()
            .to_string();
        while let Some(c) = self.next_if(|c| c.is_ascii_alphanumeric() || c == '_') {
            name.extend(c.to_lowercase());
        }
        match Keyword::try_from(name.as_str()).ok() {
            Some(keyword) => Some(Token::Keyword(keyword)),
            None => Some(Token::Ident(name)),
        }
    }

    fn next_is(&mut self, expected: char) -> bool {
        self.next_if(|c| c == expected).is_some()
    }

    fn skip_whitespace(&mut self) {
        while self.next_if(|c| c.is_whitespace()).is_some() {}
    }

    fn next_if(&mut self, predicate: impl Fn(char) -> bool) -> Option<char> {
        self.chars.peek().filter(|&&c| predicate(c))?;
        self.chars.next()
    }

    fn next_if_map<T>(&mut self, map: impl Fn(char) -> Option<T>) -> Option<T> {
        let val = self.chars.peek().and_then(|&c| map(c));
        self.chars.next();
        val
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Keyword {
    /// basic logic operators.
    And,
    Not,
    Or,

    /// Data types.
    Bool,
    Boolean,
    Int,
    Integer,
    String,
    Text,
    Float,
    Double,

    /// DDL keywords.
    Primary,
    Unique,
    Default,
    Index,
    Key,

    /// SQL keywords.
    Select,
    Insert,
    Into,
    Values,
    Create,
    Drop,
    Table,
    From,
    Where,
    Limit,
    Null,
    True,
    False,
    Nan,
    Infinity,
    Like,
}

impl From<Keyword> for Token {
    fn from(keyword: Keyword) -> Self {
        Self::Keyword(keyword)
    }
}

impl TryFrom<&str> for Keyword {
    type Error = &'static str;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        debug_assert!(value.chars().all(|c| !c.is_uppercase()));
        Ok(match value {
            "and" => Self::And,
            "or" => Self::Or,
            "not" => Self::Not,
            "select" => Self::Select,
            "create" => Self::Create,
            "insert" => Self::Insert,
            "drop" => Self::Drop,
            "from" => Self::From,
            "where" => Self::Where,
            "limit" => Self::Limit,
            "null" => Self::Null,
            "true" => Self::True,
            "false" => Self::False,
            "nan" => Self::Nan,
            "infinity" => Self::Infinity,
            "like" => Self::Like,
            "table" => Self::Table,
            "into" => Self::Into,
            "values" => Self::Values,

            "bool" => Self::Bool,
            "boolean" => Self::Boolean,
            "int" => Self::Int,
            "integer" => Self::Integer,
            "float" => Self::Float,
            "double" => Self::Double,
            "string" => Self::String,
            "text" => Self::Text,

            "primary" => Self::Primary,
            "unique" => Self::Unique,
            "default" => Self::Default,
            "index" => Self::Index,

            _ => return Err("Invalid keyword:"),
        })
    }
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Display keywords as uppercase.
        f.write_str(match self {
            Self::And => "AND",
            Self::Or => "OR",
            Self::Not => "NOT",
            Self::From => "FROM",
            Self::Insert => "INSERT",
            Self::Create => "CREATE",
            Self::Drop => "DROP",
            Self::Limit => "LIMIT",
            Self::Select => "SELECT",
            Self::Where => "WHERE",
            Self::Null => "NULL",
            Self::True => "TRUE",
            Self::False => "FALSE",
            Self::Nan => "NAN",
            Self::Infinity => "INFINITY",
            Self::Like => "LIKE",
            Self::Table => "TABLE",

            Self::Bool => "BOOL",
            Self::Boolean => "BOOLEAN",
            Self::Int => "INT",
            Self::Integer => "INTEGER",
            Self::Float => "FLOAT",
            Self::Double => "DOUBLE",
            Self::String => "STRING",
            Self::Text => "TEXT",

            Self::Primary => "PRIMARY",
            Self::Unique => "UNIQUE",
            Self::Default => "DEFAULT",
            Self::Index => "INDEX",
        })
    }
}
