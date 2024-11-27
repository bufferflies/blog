use itertools::Itertools as _;

use super::{Transaction, engine::Engine};
use crate::{
    Parser,
    error::{Error, Result},
    execution::execute::ExecutionResult,
    parser::ast,
    planner::Plan,
    types::value::{Label, Row},
};

pub struct Session<'a, E: Engine<'a>> {
    engine: &'a E,
    txn: Option<E::Transaction>,
}

impl<'a, E: Engine<'a>> Session<'a, E> {
    pub fn new(engine: &'a E) -> Self {
        Session { engine, txn: None }
    }

    pub fn execute(&mut self, statement: &str) -> Result<StatementResult> {
        Ok(match Parser::new(statement).parse()? {
            ast::Statement::Explain(statement) => self.with_txn(true, |txn| {
                Ok(StatementResult::Explain(Plan::build(*statement, txn)?))
            })?,
            statement => {
                let read_only = matches!(statement, ast::Statement::Select { .. });
                self.with_txn(read_only, |txn| {
                    Plan::build(statement, txn)?.execute(txn)?.try_into()
                })?
            }
        })
    }

    pub fn with_txn<F, T>(&mut self, read_only: bool, f: F) -> Result<T>
    where
        F: FnOnce(&mut E::Transaction) -> Result<T>,
    {
        if let Some(ref mut txn) = self.txn {
            return f(txn);
        }
        let mut txn = match read_only {
            true => self.engine.begin_read_only()?,
            false => self.engine.begin()?,
        };
        let result = f(&mut txn);
        match result {
            Ok(_) => txn.commit()?,
            Err(_) => txn.rollback()?,
        }
        result
    }
}

impl<'a, E: Engine<'a>> Drop for Session<'a, E> {
    fn drop(&mut self) {
        if let Some(txn) = self.txn.take() {
            if let Err(error) = txn.rollback() {
                log::error!("rollback failed: {}", error);
            }
        }
    }
}

#[derive(Debug)]
pub enum StatementResult {
    Explain(Plan),
    CreateTable { name: String },
    DropTable { name: String },
    Delete { count: u64 },
    Insert { count: u64 },
    Update { count: u64 },
    Select { columns: Vec<Label>, rows: Vec<Row> },
}

impl TryFrom<ExecutionResult> for StatementResult {
    type Error = Error;

    fn try_from(result: ExecutionResult) -> Result<Self> {
        Ok(match result {
            ExecutionResult::CreateTable { name } => Self::CreateTable { name },
            ExecutionResult::DropTable { name, .. } => Self::DropTable { name },
            ExecutionResult::Delete { count } => Self::Delete { count },
            ExecutionResult::Insert { count } => Self::Insert { count },
            ExecutionResult::Update { count } => Self::Update { count },
            ExecutionResult::Select { columns, rows } => Self::Select {
                columns,
                rows: rows.try_collect()?,
            },
        })
    }
}
