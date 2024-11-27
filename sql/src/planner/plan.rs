use std::{cmp::Ordering, collections::HashMap};

use itertools::Itertools as _;
use serde::{Deserialize, Serialize};

use super::Planner;
use crate::{
    engine::{Catalog, Transaction},
    error::Result,
    execution::{self, execute::ExecutionResult},
    parser::ast,
    types::{expression::Expression, schema::Table, value::Label},
};

#[derive(Debug, Deserialize, Serialize)]
pub enum Plan {
    CreateTable {
        schema: Table,
    },
    Insert {
        table: Table,
        column_map: Option<HashMap<usize, usize>>,
        source: Node,
    },
    Select(Node),
}

impl Plan {
    pub fn build(statement: ast::Statement, catalog: &impl Catalog) -> Result<Self> {
        Planner::new(catalog).build(statement)
    }

    pub fn execute(self, txn: &(impl Transaction + Catalog)) -> Result<ExecutionResult> {
        execution::execute_plan(self, txn, txn)
    }

    pub fn optimize(self) -> Result<Self> {
        Ok(self)
    }
}

impl std::fmt::Display for Plan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Plan::CreateTable { schema } => write!(f, "CreateTable {}", schema.name),
            Plan::Insert { table, source, .. } => {
                write!(f, "Insert {}", table.name)?;
                source.format(f, "", false, true)
            }
            Plan::Select(root) => root.format(f, "", true, true),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Node {
    Aggregate {
        source: Box<Node>,
        group_by: Vec<Expression>,
        aggregates: Vec<Aggregate>,
    },

    Filter {
        source: Box<Node>,
        predicate: Expression,
    },

    Offset {
        source: Box<Node>,
        offset: usize,
    },

    Limit {
        source: Box<Node>,
        limit: usize,
    },
    Order {
        source: Box<Node>,
        key: Vec<(Expression, Direction)>,
    },

    Scan {
        table: Table,
        filter: Option<Expression>,
    },
    Values {
        rows: Vec<Vec<Expression>>,
    },
}

impl Node {
    pub fn format(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        prefix: &str,
        root: bool,
        last_child: bool,
    ) -> std::fmt::Result {
        if !root {
            writeln!(f)?;
        }
        let prefix = if !last_child {
            write!(f, "{prefix}├─ ")?;
            format!("{prefix}| ")
        } else if !root {
            write!(f, "{prefix}└─ ")?;
            format!("{prefix}   ")
        } else {
            write!(f, "{prefix}")?;
            prefix.to_string()
        };
        match self {
            Self::Aggregate {
                source,
                group_by,
                aggregates,
            } => {
                let aggregates = group_by
                    .iter()
                    .map(|group_by| group_by.format(self))
                    .chain(aggregates.iter().map(|agg| agg.format(source)))
                    .join(",");
                write!(f, "Aggregate: {}", aggregates)?;
                source.format(f, &prefix, false, true)?;
            }

            Self::Order {
                source,
                key: orders,
            } => {
                let orders = orders
                    .iter()
                    .map(|(expr, dir)| format!("{} {dir}", expr.format(source)))
                    .join(",");
                write!(f, "Order: {}", orders)?;
                source.format(f, &prefix, root, last_child)?;
            }
            Self::Filter { source, predicate } => {
                write!(f, "Filter: {}", predicate.format(source))?;
                source.format(f, &prefix, false, true)?;
            }
            Self::Offset { source, offset } => {
                write!(f, "Offset: {}", offset)?;
                source.format(f, &prefix, false, true)?;
            }
            Self::Limit { source, limit } => {
                write!(f, "Limit: {}", limit)?;
                source.format(f, &prefix, false, true)?;
            }
            Self::Scan { table, filter } => {
                write!(f, "Scan: {}", table.name)?;
                if let Some(filter) = filter {
                    write!(f, " Filter: {}", filter.format(self))?;
                }
            }
            Self::Values { rows, .. } => {
                write!(f, "Values ")?;
                match rows.len() {
                    1 if rows[0].is_empty() => write!(f, "blank row")?,
                    1 => write!(f, "{}", rows[0].iter().map(|e| e.format(self)).join(","))?,
                    n => write!(f, "{n} rows")?,
                }
            }
        }
        Ok(())
    }

    pub fn columns(&self) -> usize {
        match self {
            Node::Aggregate {
                aggregates,
                group_by,
                ..
            } => aggregates.len() + group_by.len(),
            Node::Filter { source, .. } => source.columns(),
            Node::Scan { table, .. } => table.columns.len(),
            Node::Values { rows } => rows.first().map(|r| r.len()).unwrap_or_default(),
            _ => 0,
        }
    }

    pub fn column_label(&self, index: usize) -> Label {
        match self {
            Node::Scan { table, .. } => {
                Label::Qualified(table.name.clone(), table.columns[index].name.clone())
            }
            _ => Label::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Direction {
    Ascending,
    Descending,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Ascending => write!(f, "asc"),
            Direction::Descending => write!(f, "desc"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Aggregate {
    Average(Expression),
    Count(Expression),
    Max(Expression),
    Min(Expression),
    Sum(Expression),
}

impl Aggregate {
    pub fn format(&self, source: &Node) -> String {
        match self {
            Self::Average(expr) => format!("avg({})", expr.format(source)),
            Self::Count(expr) => format!("count({})", expr.format(source)),
            Self::Max(expr) => format!("max({})", expr.format(source)),
            Self::Min(expr) => format!("min({})", expr.format(source)),
            Self::Sum(expr) => format!("sum({})", expr.format(source)),
        }
    }
}
