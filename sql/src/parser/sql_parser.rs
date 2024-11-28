use super::{
    Parser,
    ast::{self, Column},
    lexer::{Keyword, Token},
};
use crate::{errinput, error::Result, types::schema::DataType};

impl<'a> Parser<'a> {
    pub fn parse_statement(&mut self) -> Result<ast::Statement> {
        let Some(token) = self.peek()? else {
            return errinput!("Unexpected end of input");
        };
        match token {
            Token::Keyword(Keyword::Create) => self.parse_create_table(),
            Token::Keyword(Keyword::Select) => self.parse_select(),
            Token::Keyword(Keyword::Insert) => self.parse_insert(),
            token => errinput!("parse statement met unexpected end of input:{token:?}"),
        }
    }

    fn parse_select(&mut self) -> Result<ast::Statement> {
        Ok(ast::Statement::Select {
            select: self.parse_select_clause()?,
            from: self.parse_from_clause()?,
            r#where: self.parse_where_clause()?,
            limit: self
                .next_is(Keyword::Limit.into())
                .then(|| self.parse_expression())
                .transpose()?,
        })
    }

    fn parse_where_clause(&mut self) -> Result<Option<ast::Expression>> {
        if !self.next_is(Keyword::Where.into()) {
            return Ok(None);
        }
        Ok(Some(self.parse_expression()?))
    }

    fn parse_from_clause(&mut self) -> Result<Vec<ast::From>> {
        if !self.next_is(Keyword::From.into()) {
            return Ok(Vec::new());
        }
        let mut from = Vec::new();
        loop {
            let table = self.parse_from_table()?;
            from.push(table);
            if !self.next_is(Token::Comma) {
                break;
            }
        }
        Ok(from)
    }

    fn parse_from_table(&mut self) -> Result<ast::From> {
        let name = self.next_ident()?;
        let mut alias = None;
        if self.next_is(Keyword::As.into()) || matches!(self.peek()?, Some(Token::Ident(_))) {
            alias = Some(self.next_ident()?);
        }
        Ok(ast::From::Table { name, alias })
    }

    fn parse_select_clause(&mut self) -> Result<Vec<(ast::Expression, Option<String>)>> {
        if !self.next_is(Keyword::Select.into()) {
            return Ok(Vec::new());
        }
        let mut select = vec![];
        loop {
            let expr = self.parse_expression()?;
            let mut label = None;
            if self.next_is(Keyword::As.into()) || matches!(self.peek()?, Some(Token::Ident(_))) {
                if expr == ast::Expression::All {
                    return errinput!("Cannot select all and a column");
                }
                label = Some(self.next_ident()?);
            }
            select.push((expr, label));
            if !self.next_is(Token::Comma) {
                break;
            }
        }
        Ok(select)
    }

    fn parse_insert(&mut self) -> Result<ast::Statement> {
        self.expect(Keyword::Insert.into())?;
        self.expect(Keyword::Into.into())?;
        let table_name = self.next_ident()?;
        let mut columns = None;
        if self.next_is(Token::OpenParen) {
            let columns = columns.insert(Vec::new());
            loop {
                columns.push(self.next_ident()?);
                if !self.next_is(Token::Comma) {
                    break;
                }
            }
            self.expect(Token::CloseParen.into())?;
        }

        self.expect(Keyword::Values.into())?;

        let mut values = Vec::new();
        loop {
            let mut rows = Vec::new();
            self.expect(Token::OpenParen)?;
            loop {
                rows.push(self.parse_expression()?);
                if !self.next_is(Token::Comma) {
                    break;
                }
            }
            self.expect(Token::CloseParen)?;
            values.push(rows);
            if !self.next_is(Token::Comma) {
                break;
            }
        }

        Ok(ast::Statement::Insert {
            table_name,
            columns,
            values,
        })
    }

    fn parse_create_table(&mut self) -> Result<ast::Statement> {
        self.expect(Keyword::Create.into())?;
        self.expect(Keyword::Table.into())?;
        let table_name = self.next_ident()?;
        self.expect(Token::OpenParen)?;
        let mut columns = Vec::new();
        loop {
            let column = self.parse_column()?;
            columns.push(column);
            if !self.next_is(Token::Comma) {
                break;
            }
        }
        self.expect(Token::CloseParen)?;
        Ok(ast::Statement::CrateTable {
            table_name,
            columns,
        })
    }

    fn parse_column(&mut self) -> Result<Column> {
        let column_name = self.next_ident()?;
        let datatype = match self.next()? {
            Token::Keyword(Keyword::Bool | Keyword::Boolean) => DataType::Boolean,
            Token::Keyword(Keyword::Int | Keyword::Integer) => DataType::Integer,
            Token::Keyword(Keyword::Float | Keyword::Double) => DataType::Float,
            Token::Keyword(Keyword::String) => DataType::String,
            token => return errinput!("parse column met unexpected token:{token:?}"),
        };
        let mut column = Column {
            name: column_name,
            datatype,
            nullable: None,
            default: None,
            unique: false,
            primary_key: false,
        };
        while let Some(keyword) = self.next_if_keyword() {
            match keyword {
                Keyword::Primary => {
                    self.expect(Keyword::Key.into())?;
                    column.primary_key = true;
                }
                Keyword::Null => {
                    if column.nullable.is_some() {
                        return errinput!("Nullable already set for column:{}", column.name);
                    }
                    column.nullable = Some(true);
                }
                Keyword::Unique => {
                    column.unique = true;
                }
                Keyword::Not => {
                    self.expect(Keyword::Null.into())?;
                    if column.nullable.is_some() {
                        return errinput!("Nullable already set for column:{}", column.name);
                    }
                    column.nullable = Some(false);
                }

                Keyword::Default => {
                    let value = self.parse_expression()?;
                    column.default = Some(value);
                }
                _ => return errinput!("Unexpected keyword:{keyword:?}"),
            }
        }
        Ok(column)
    }
}
