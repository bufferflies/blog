use super::{
    Lexer, ast,
    lexer::{Keyword, Token},
};
use crate::{
    errinput,
    error::{Error, Result},
};

pub struct Parser<'a> {
    pub lexer: std::iter::Peekable<Lexer<'a>>,
}

type Precedence = u8;

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: Lexer::new(input).peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<ast::Statement> {
        let statement = self.parse_statement()?;
        if self.next_is(Token::Semicolon) {
            return Error::InvalidInput("Unexpected end of input".to_string()).into();
        }
        if let Some(token) = self.lexer.next().transpose()? {
            return Error::InvalidInput(format!("Unexpected end of input:{:?}", token)).into();
        }
        Ok(statement)
    }

    fn parse_statement(&mut self) -> Result<ast::Statement> {
        let Some(token) = self.peek()? else {
            return Error::InvalidInput("Unexpected end of input".to_string()).into();
        };
        match token {
            Token::Keyword(Keyword::Select) => self.parse_select(),
            token => Error::InvalidInput(format!("Unexpected end of input:{:?}", token)).into(),
        }
    }

    fn parse_select(&mut self) -> Result<ast::Statement> {
        let statement = {
            ast::Statement::Select {
                select: self.parse_select_clause()?,
                from: self.parse_from_clause()?,
                limit: self
                    .next_is(Keyword::Limit.into())
                    .then(|| self.parse_expression())
                    .transpose()?,
            }
        };
        Ok(statement)
    }

    fn parse_from_clause(&mut self) -> Result<Vec<ast::TableName>> {
        if !self.next_is(Keyword::From.into()) {
            return Ok(Vec::new());
        }
        let mut tables = Vec::new();
        loop {
            let table = self.parse_from_table()?;
            tables.push(table);
            if !self.next_is(Token::Comma) {
                break;
            }
        }
        Ok(tables)
    }

    fn parse_from_table(&mut self) -> Result<ast::TableName> {
        self.next_ident()
    }

    fn next_ident(&mut self) -> Result<String> {
        match self.next()? {
            Token::Ident(ident) => Ok(ident),
            token => Error::InvalidInput(format!("Unexpected token:{:?}", token)).into(),
        }
    }

    fn parse_select_clause(&mut self) -> Result<Vec<(ast::Expression, Option<String>)>> {
        if !self.next_is(Keyword::Select.into()) {
            return Ok(Vec::new());
        }
        let mut select = Vec::new();
        loop {
            let expr = self.parse_expression()?;
            let label = None;
            select.push((expr, label));
            if !self.next_is(Token::Comma) {
                break;
            }
        }
        Ok(select)
    }

    pub fn parse_expression(&mut self) -> Result<ast::Expression> {
        self.parse_expression_at(0)
    }

    fn parse_expression_at(&mut self, min_precedence: Precedence) -> Result<ast::Expression> {
        let mut lhs = if let Some(prefix) = self.parse_prefix_operator(min_precedence) {
            let at_precedence = prefix.precedence() + prefix.associativity();
            prefix.build(self.parse_expression_at(at_precedence)?)
        } else {
            self.parse_expression_atom()?
        };

        // while let Some(postfix)=self.parse_postfix_operator(min_precedence)?{
        //     lhs=postfix.build(lhs);
        // }
        // Apply any binary infix operators, parsing the right-hand operand.
        while let Some(infix) = self.parse_infix_operator(min_precedence) {
            let at_precedence = infix.precedence() + infix.associativity();
            let rhs = self.parse_expression_at(at_precedence)?;
            lhs = infix.build(lhs, rhs);
        }
        // Apply any postfix operators after the binary operator. Consider e.g.
        // 1 + NULL IS NULL.
        // while let Some(postfix) = self.parse_postfix_operator(min_precedence)? {
        //     lhs = postfix.build(lhs)
        // }
        Ok(lhs)
    }

    // fn parse_postfix_operator(&mut self,min_precedence:Precedence) ->
    // Result<Option<PostfixOperator>>{    Ok(None)
    // }

    fn parse_infix_operator(&mut self, min_precedence: Precedence) -> Option<InfixOperator> {
        self.next_if_map(|token| {
            let operator = match token {
                Token::Keyword(Keyword::And) => InfixOperator::And,
                Token::Keyword(Keyword::Or) => InfixOperator::Or,

                Token::Equal => InfixOperator::Equal,
                Token::NotEqual => InfixOperator::NotEqual,
                Token::GreaterThan => InfixOperator::GreaterThan,
                Token::GreaterThanOrEqual => InfixOperator::GreaterThanOrEqual,
                Token::LessThanOrEqual => InfixOperator::LessThanOrEqual,
                Token::LessThan => InfixOperator::LessThan,

                Token::Slash => InfixOperator::Divide,
                Token::Asterisk => InfixOperator::Multiply,
                Token::Plus => InfixOperator::Add,
                Token::Minus => InfixOperator::Subtract,
                Token::Percent => InfixOperator::Remainder,
                Token::Exponential => InfixOperator::Exponential,
                Token::Keyword(Keyword::Like) => InfixOperator::Like,
                _ => return None,
            };
            Some(operator).filter(|op| op.precedence() >= min_precedence)
        })
    }

    fn parse_prefix_operator(&mut self, min_precedence: Precedence) -> Option<PrefixOperator> {
        self.next_if_map(|token| {
            let operator = match token {
                Token::Keyword(Keyword::Not) => PrefixOperator::Not,
                Token::Minus => PrefixOperator::Minus,
                Token::Plus => PrefixOperator::Plus,
                _ => return None,
            };
            Some(operator).filter(|op| op.precedence() >= min_precedence)
        })
    }

    fn parse_expression_atom(&mut self) -> Result<ast::Expression> {
        let token = match self.next()? {
            Token::Asterisk => ast::Expression::All,
            Token::Number(str) if str.chars().all(|c| c.is_ascii_digit()) => {
                ast::Literal::Integer(str.parse()?).into()
            }
            Token::Number(str) => ast::Literal::Float(str.parse()?).into(),
            Token::String(s) => ast::Literal::String(s).into(),
            Token::Ident(function_name) if self.next_is(Token::OpenParen) => {
                let mut args = Vec::new();
                while !self.next_is(Token::CloseParen) {
                    if !args.is_empty() {
                        self.expect(Token::Comma)?;
                    }
                    args.push(self.parse_expression()?);
                }
                ast::Expression::Function(function_name, args).into()
            }
            Token::Keyword(Keyword::Null) => ast::Literal::Null.into(),
            Token::Keyword(Keyword::True) => ast::Literal::Boolean(true).into(),
            Token::Keyword(Keyword::False) => ast::Literal::Boolean(false).into(),
            Token::Keyword(Keyword::Infinity) => ast::Literal::Float(f64::INFINITY).into(),
            Token::Keyword(Keyword::Nan) => ast::Literal::Float(f64::NAN).into(),
            token => return errinput!("expected expression atom, found {token}"),
        };
        Ok(token)
    }

    fn expect(&mut self, expect: Token) -> Result<()> {
        let token = self.next()?;
        if token != expect {
            return errinput!("expected token {expect}, found {token}");
        }
        Ok(())
    }

    fn next_is(&mut self, token: Token) -> bool {
        self.next_if(|t| *t == token).is_some()
    }

    fn next_if(&mut self, predicate: impl Fn(&Token) -> bool) -> Option<Token> {
        self.peek().unwrap_or(None).filter(|t| predicate(t))?;
        self.next().ok()
    }

    fn next_if_map<T>(&mut self, predicate: impl Fn(&Token) -> Option<T>) -> Option<T> {
        let out = self.peek().unwrap_or(None).map(predicate)?;
        if out.is_some() {
            self.next().ok();
        }
        out
    }

    fn next(&mut self) -> Result<Token> {
        self.lexer
            .next()
            .transpose()?
            .ok_or_else(|| errinput!("unexpected end of input"))
    }

    fn peek(&mut self) -> Result<Option<&Token>> {
        self.lexer
            .peek()
            .map(|r| r.as_ref().map_err(|err| err.clone()))
            .transpose()
    }
}

/// Prefix operators.
enum PrefixOperator {
    Minus, // -a
    Not,   // NOT a
    Plus,  // +a
}

const LEFT_ASSOCIATIVE: Precedence = 1;
const RIGHT_ASSOCIATIVE: Precedence = 0;

impl PrefixOperator {
    /// Returns the precedence of the operator.
    fn precedence(&self) -> Precedence {
        match self {
            Self::Not => 3,
            Self::Minus | Self::Plus => 10,
        }
    }

    fn associativity(&self) -> Precedence {
        RIGHT_ASSOCIATIVE
    }

    fn build(self, rhs: ast::Expression) -> ast::Expression {
        match self {
            Self::Not => ast::Operator::Not(Box::new(rhs)).into(),
            Self::Minus => ast::Operator::Negate(Box::new(rhs)).into(),
            Self::Plus => ast::Operator::Identity(Box::new(rhs)).into(),
        }
    }
}

enum InfixOperator {
    And,                // a AND b
    Or,                 // a OR b
    Equal,              // a = b
    NotEqual,           // a != b
    GreaterThan,        // a > b
    GreaterThanOrEqual, // a >= b
    LessThan,           // a < b
    LessThanOrEqual,    // a <= b
    Add,                // a + b
    Subtract,           // a - b
    Multiply,           // a * b
    Divide,             // a / b
    Remainder,          // a % b
    Exponential,        // a ^ b

    Like, // a LIKE b
}

impl InfixOperator {
    fn precedence(&self) -> Precedence {
        match self {
            Self::Or => 1,
            Self::And => 2,
            Self::Equal | Self::NotEqual | Self::Like => 4,
            Self::GreaterThan
            | Self::LessThan
            | Self::GreaterThanOrEqual
            | Self::LessThanOrEqual => 5,
            Self::Add | Self::Subtract => 6,
            Self::Multiply | Self::Divide | Self::Remainder => 7,
            Self::Exponential => 8,
        }
    }

    fn associativity(&self) -> Precedence {
        LEFT_ASSOCIATIVE
    }

    fn build(self, lhs: ast::Expression, rhs: ast::Expression) -> ast::Expression {
        match self {
            Self::And => ast::Operator::And(Box::new(lhs), Box::new(rhs)).into(),
            Self::Or => ast::Operator::Or(Box::new(lhs), Box::new(rhs)).into(),
            Self::Equal => ast::Operator::Equal(Box::new(lhs), Box::new(rhs)).into(),
            Self::NotEqual => ast::Operator::NotEqual(Box::new(lhs), Box::new(rhs)).into(),
            Self::GreaterThan => ast::Operator::GreaterThan(Box::new(lhs), Box::new(rhs)).into(),
            Self::GreaterThanOrEqual => {
                ast::Operator::GreaterThanOrEqual(Box::new(lhs), Box::new(rhs)).into()
            }
            Self::LessThan => ast::Operator::LessThan(Box::new(lhs), Box::new(rhs)).into(),
            Self::LessThanOrEqual => {
                ast::Operator::LessThanOrEqual(Box::new(lhs), Box::new(rhs)).into()
            }
            Self::Add => ast::Operator::Add(Box::new(lhs), Box::new(rhs)).into(),
            Self::Subtract => ast::Operator::Subtract(Box::new(lhs), Box::new(rhs)).into(),
            Self::Multiply => ast::Operator::Multiply(Box::new(lhs), Box::new(rhs)).into(),
            Self::Divide => ast::Operator::Divide(Box::new(lhs), Box::new(rhs)).into(),
            Self::Remainder => ast::Operator::Remainder(Box::new(lhs), Box::new(rhs)).into(),
            Self::Exponential => ast::Operator::Exponential(Box::new(lhs), Box::new(rhs)).into(),

            Self::Like => ast::Operator::Like(Box::new(lhs), Box::new(rhs)).into(),
        }
    }
}
