use std::collections::{HashMap, HashSet};

use super::plan::{Node, Plan};
use crate::{
    engine::Catalog,
    errinput,
    error::Result,
    parser::ast::{self, ColumnName, TableName},
    types::{
        expression::Expression,
        schema::{Column, Table},
        value::{Label, Value},
    },
};

pub struct Planner<'a, C: Catalog> {
    catalog: &'a C,
}

impl<'a, C: Catalog> Planner<'a, C> {
    /// creates a new planner
    pub fn new(catalog: &'a C) -> Self {
        Self { catalog }
    }

    pub fn build(&mut self, statement: ast::Statement) -> Result<Plan> {
        use ast::Statement::*;
        match statement {
            CrateTable {
                table_name,
                columns,
            } => self.build_create_table(table_name, columns),
            Insert {
                table_name,
                columns,
                values,
            } => self.build_insert(table_name, columns, values),
            Select {
                select,
                from,
                r#where,
                limit,
            } => self.build_select(select, from, r#where, limit),
            _ => errinput!("not support this statement:{statement:?}"),
        }
    }

    fn build_select(
        &self,
        mut select: Vec<(ast::Expression, Option<String>)>,
        from: Vec<ast::From>,
        r#where: Option<ast::Expression>,
        limit: Option<ast::Expression>,
    ) -> Result<Plan> {
        let mut scope = Scope::new();
        let mut node = if !from.is_empty() {
            self.build_from_clause(from, &mut scope)?
        } else {
            Node::Values { rows: vec![vec![]] }
        };
        if select.contains(&(ast::Expression::All, None)) {
            if node.columns() == 0 {
                return errinput!("no columns in the table");
            }
            if select.len() > 1 {
                select = select
                    .into_iter()
                    .flat_map(|(expr, alias)| match expr {
                        ast::Expression::All => itertools::Either::Left(
                            (0..node.columns()).map(|i| (node.column_label(i).into(), None)),
                        ),
                        expr => itertools::Either::Right(std::iter::once((expr, alias))),
                    })
                    .collect();
            }
        }

        if let Some(r#where) = r#where {
            let predicate = Self::build_expression(r#where, &scope)?;
            node = Node::Filter {
                source: Box::new(node),
                predicate,
            }
        }

        if select.as_slice() != [(ast::Expression::All, None)] {
            let child_scope = scope.project(&select);
            let mut expressions = Vec::with_capacity(select.len());
            let mut aliases = Vec::with_capacity(select.len());
            for (expression, alias) in select {
                expressions.push(Self::build_expression(expression, &scope)?);
                aliases.push(Label::from(alias));
            }
            scope = child_scope;
            node = Node::Projection {
                source: Box::new(node),
                expressions,
                aliases,
            }
        }

        if let Some(limit) = limit {
            let limit = match Self::evaluate_constant(limit)? {
                Value::Integer(limit) if limit >= 0 => limit as usize,
                limit => return errinput!("invalid limit:{limit}"),
            };
            node = Node::Limit {
                source: Box::new(node),
                limit,
            }
        }

        Ok(Plan::Select(node))
    }

    fn build_from_clause(&self, from: Vec<ast::From>, scope: &mut Scope) -> Result<Node> {
        let mut items = from.into_iter();
        let node = match items.next() {
            Some(from) => self.build_from(from, scope)?,
            None => return errinput!("no from items given"),
        };

        Ok(node)
    }

    fn build_from(&self, from: ast::From, parent_scope: &mut Scope) -> Result<Node> {
        let mut scope = Scope::new();
        let node = match from {
            ast::From::Table { name, alias } => {
                let table = self.catalog.must_get_table(&name)?;
                scope.add_table(&table, alias.as_deref())?;
                Node::Scan {
                    table,
                    filter: None,
                }
            }
            _ => return errinput!("not support join {from:?}"),
        };
        parent_scope.merge(scope)?;
        Ok(node)
    }

    fn build_insert(
        &self,
        table_name: String,
        columns: Option<Vec<String>>,
        values: Vec<Vec<ast::Expression>>,
    ) -> Result<Plan> {
        let table = self.catalog.get_table(&table_name)?.unwrap();
        let mut column_map = None;
        if let Some(columns) = columns {
            let column_map = column_map.insert(HashMap::new());
            for (vidx, name) in columns.into_iter().enumerate() {
                let Some(cidx) = table.columns.iter().position(|c| c.name == name) else {
                    return errinput!("column not found:{name}");
                };
                if column_map.insert(vidx, cidx).is_some() {
                    return errinput!("duplicate column:{name}");
                };
            }
        }
        let scope = Scope::new();
        let values = values
            .into_iter()
            .map(|exprs| {
                exprs
                    .into_iter()
                    .map(|expr| Self::build_expression(expr, &scope))
                    .collect()
            })
            .collect::<Result<_>>()?;

        Ok(Plan::Insert {
            table,
            column_map,
            source: Node::Values { rows: values },
        })
    }

    fn build_create_table(&self, table_name: String, columns: Vec<ast::Column>) -> Result<Plan> {
        let Some(primary_key) = columns.iter().position(|c| c.primary_key) else {
            return errinput!("no primary key for this table:{table_name}");
        };
        if columns.iter().filter(|c| c.primary_key).count() > 1 {
            return errinput!("multiple primary key for this table:{table_name}");
        };
        let columns = columns
            .into_iter()
            .map(|c| {
                let nullable = c.nullable.unwrap_or(!c.primary_key);
                Ok(Column {
                    name: c.name,
                    data_type: c.datatype,
                    nullable,
                    default: match c.default {
                        Some(expr) => Some(Self::evaluate_constant(expr)?),
                        None if nullable => Some(Value::Null),
                        None => None,
                    },
                    unique: c.unique,
                })
            })
            .collect::<Result<_>>()?;
        Ok(Plan::CreateTable {
            schema: Table {
                name: table_name,
                primary_key,
                columns,
            },
        })
    }

    fn evaluate_constant(expr: ast::Expression) -> Result<Value> {
        Self::build_expression(expr, &Scope::new())?.evaluate(None)
    }

    pub fn build_expression(expr: ast::Expression, scope: &Scope) -> Result<Expression> {
        use Expression::*;
        let build_fn = |expr: Box<ast::Expression>| -> Result<Box<Expression>> {
            Ok(Box::new(Self::build_expression(*expr, scope)?))
        };
        let ret = match expr {
            ast::Expression::Literal(lit) => Constant(match lit {
                ast::Literal::Null => Value::Null,
                ast::Literal::Boolean(b) => Value::Boolean(b),
                ast::Literal::Integer(i) => Value::Integer(i),
                ast::Literal::Float(f) => Value::Float(f),
                ast::Literal::String(s) => Value::String(s),
            }),
            ast::Expression::Column(table_name, column_name) => {
                Column(scope.lookup_column(table_name.as_deref(), column_name.as_str())?)
            }
            ast::Expression::Function(function_name, mut expression) => {
                match (function_name.as_str(), expression.len()) {
                    ("sqrt", 1) => SquareRoot(build_fn(Box::new(expression.remove(0)))?),
                    (name, n) => {
                        return errinput!("unknown function {name} with {n} arguments");
                    }
                }
            }
            ast::Expression::Operator(op) => match op {
                ast::Operator::And(lhs, rhs) => And(build_fn(lhs)?, build_fn(rhs)?),
                ast::Operator::Or(lhs, rhs) => Or(build_fn(lhs)?, build_fn(rhs)?),
                ast::Operator::Not(expr) => Not(build_fn(expr)?),

                ast::Operator::Equal(lhs, rhs) => Equal(build_fn(lhs)?, build_fn(rhs)?),
                ast::Operator::NotEqual(lhs, rhs) => {
                    Not(Equal(build_fn(lhs)?, build_fn(rhs)?).into())
                }
                ast::Operator::GreaterThan(lhs, rhs) => GreaterThan(build_fn(lhs)?, build_fn(rhs)?),
                ast::Operator::LessThan(lhs, rhs) => LessThan(build_fn(lhs)?, build_fn(rhs)?),
                ast::Operator::GreaterThanOrEqual(lhs, rhs) => Or(
                    GreaterThan(build_fn(lhs.clone())?, build_fn(rhs.clone())?).into(),
                    Equal(build_fn(lhs)?, build_fn(rhs)?).into(),
                ),
                ast::Operator::LessThanOrEqual(lhs, rhs) => Or(
                    LessThan(build_fn(lhs.clone())?, build_fn(rhs.clone())?).into(),
                    Equal(build_fn(lhs)?, build_fn(rhs)?).into(),
                ),

                ast::Operator::Add(lhs, rhs) => Add(build_fn(lhs)?, build_fn(rhs)?),
                ast::Operator::Subtract(lhs, rhs) => Subtract(build_fn(lhs)?, build_fn(rhs)?),
                ast::Operator::Multiply(lhs, rhs) => Multiply(build_fn(lhs)?, build_fn(rhs)?),
                ast::Operator::Divide(lhs, rhs) => Divide(build_fn(lhs)?, build_fn(rhs)?),
                ast::Operator::Remainder(lhs, rhs) => Remainder(build_fn(lhs)?, build_fn(rhs)?),
                ast::Operator::Exponential(lhs, rhs) => Exponential(build_fn(lhs)?, build_fn(rhs)?),
                ast::Operator::Identity(expr) => Identity(build_fn(expr)?),
                ast::Operator::Negate(expr) => Negate(build_fn(expr)?),

                ast::Operator::Like(lhs, rhs) => Like(build_fn(lhs)?, build_fn(rhs)?),
                op => return errinput!("unsupported operator:{op:?}"),
            },
            e => return errinput!("unsupported expression:{e:?}"),
        };
        Ok(ret)
    }
}

pub struct Scope {
    columns: Vec<Label>,
    tables: HashSet<String>,
    /// <table_name, column_name> --> column_index
    qualified: HashMap<(TableName, ColumnName), usize>,

    /// table_name.column_nam --> column_index
    unqualified: HashMap<String, Vec<usize>>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            tables: HashSet::new(),
            qualified: HashMap::new(),
            unqualified: HashMap::new(),
        }
    }

    fn spawn(&self) -> Self {
        let mut child = Scope::new();
        child.tables = self.tables.clone();
        child
    }

    fn project(&self, expression: &[(ast::Expression, Option<String>)]) -> Self {
        let mut child = self.spawn();
        for (expr, alias) in expression {
            let mut label = Label::None;
            if let Some(alias) = alias {
                label = Label::Unqualified(alias.clone());
            } else if let ast::Expression::Column(table, column) = expr {
                if let Ok(index) = self.lookup_column(table.as_deref(), column.as_str()) {
                    label = self.columns[index].clone();
                }
            }
            child.add_column(label);
        }
        child
    }

    fn merge(&mut self, scope: Scope) -> Result<()> {
        for table in scope.tables {
            if self.tables.contains(&table) {
                return errinput!("merge met duplicate table:{table}");
            }
            self.tables.insert(table);
        }
        for label in scope.columns {
            self.add_column(label);
        }
        Ok(())
    }

    fn add_table(&mut self, table: &Table, alias: Option<&str>) -> Result<()> {
        let table_name = alias.unwrap_or(&table.name);
        if self.tables.contains(table_name) {
            return errinput!("add met duplicate table:{table_name}");
        }
        for column in &table.columns {
            self.add_column(Label::Qualified(
                table_name.to_string(),
                column.name.clone(),
            ));
        }
        self.tables.insert(table_name.to_string());
        Ok(())
    }

    fn add_column(&mut self, label: Label) -> usize {
        let index = self.columns.len();
        if let Label::Qualified(table, column) = &label {
            self.qualified
                .insert((table.clone(), column.clone()), index);
        }
        if let Label::Qualified(_, column) | Label::Unqualified(column) = &label {
            self.unqualified
                .entry(column.clone())
                .or_default()
                .push(index)
        }
        self.columns.push(label);
        index
    }

    fn lookup_column(&self, table: Option<&str>, name: &str) -> Result<usize> {
        let fmtname = || {
            table
                .map(|table| format!("{table}.{name}"))
                .unwrap_or(name.to_string())
        };
        if self.columns.is_empty() {
            return errinput!("expression must be constant, found column {}", fmtname());
        }
        if let Some(table) = table {
            if !self.tables.contains(table) {
                return errinput!("unknown table:{table}");
            }
            if let Some(index) = self.qualified.get(&(table.to_string(), name.to_string())) {
                return Ok(*index);
            }
        } else if let Some(indexes) = self.unqualified.get(name) {
            if indexes.len() > 1 {
                return errinput!("ambiguous column:{name}");
            }
            return Ok(indexes[0]);
        }
        errinput!("unknown column:{}", fmtname())
    }
}
