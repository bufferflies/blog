use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::Planner;
use crate::{
    engine::{Catalog, Transaction},
    error::Result,
    execution::{self, execute::ExecutionResult},
    parser::ast,
    types::{expression::Expression, schema::Table, value::Label},
};

#[derive(Debug,Deserialize,Serialize)]
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
    Node {
        columns: Vec<Label>,
    },
    Values {
        rows: Vec<Vec<Expression>>,
    },
}

impl Node {
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

#[derive(Debug,Clone,PartialEq,Serialize, Deserialize)]
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

#[derive(Debug,Clone,PartialEq,Serialize, Deserialize)]
pub enum Aggregate {
    Average(Expression),
    Count(Expression),
    Max(Expression),
    Min(Expression),
    Sum(Expression),
}
