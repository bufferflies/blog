use crate::{
    engine::Transaction,
    error::Result,
    types::{expression::Expression, schema::Table, value::Rows},
};

pub fn scan(txn: &impl Transaction, table: Table, filter: Option<Expression>) -> Result<Rows> {
    Ok(Box::new(txn.scan(&table.name, filter)?))
}
pub fn nothing() -> Rows {
    Box::new(std::iter::empty())
}

pub fn values(rows: Vec<Vec<Expression>>) -> Rows {
    Box::new(
        rows.into_iter()
            .map(|row| row.into_iter().map(|expr| expr.evaluate(None)).collect()),
    )
}
