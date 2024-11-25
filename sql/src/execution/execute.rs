use super::{source, transform, write};
use crate::{
    engine::{Catalog, Transaction},
    errinput,
    error::Result,
    planner::{Node, Plan},
    types::value::{Label, Rows},
};

pub fn execute_plan(
    plan: Plan,
    catalog: &impl Catalog,
    txn: &impl Transaction,
) -> Result<ExecutionResult> {
    Ok(match plan {
        Plan::CreateTable { schema } => {
            let name = schema.name.clone();
            catalog.create_table(schema)?;
            ExecutionResult::CreateTable { name }
        }
        Plan::Insert {
            table,
            column_map,
            source,
        } => {
            let source = execute(source, txn)?;
            let count = write::insert(txn, table, column_map, source)?;
            ExecutionResult::Insert { count }
        }
        Plan::Select(root) => {
            let columns = (0..root.columns()).map(|i| root.column_label(i)).collect();
            let rows = execute(root, txn)?;
            ExecutionResult::Select { rows, columns }
        }
        _ => errinput!("unsupported plan"),
    })
}

pub fn execute(node: Node, txn: &impl Transaction) -> Result<Rows> {
    Ok(match node {
        Node::Filter { source, predicate } => {
            let source = execute(*source, txn)?;
            transform::filter(source, predicate)
        }
        Node::Offset { source, offset } => {
            let mut source = execute(*source, txn)?;
            transform::offset(source, offset)
        }
        Node::Order {
            source,
            key: orders,
        } => {
            let source = execute(*source, txn)?;
            transform::order(source, orders)?
        }
        Node::Limit { source, limit } => {
            let source = execute(*source, txn)?;
            transform::limit(source, limit)
        }
        Node::Values { rows } => source::values(rows),
        _ => {
            log::error!("unsupported node:{node:?}");
            todo!()
        }
    })
}

/// A plan execution result.
pub enum ExecutionResult {
    CreateTable { name: String },
    DropTable { name: String, existed: bool },
    Delete { count: u64 },
    Insert { count: u64 },
    Update { count: u64 },
    Select { rows: Rows, columns: Vec<Label> },
}
