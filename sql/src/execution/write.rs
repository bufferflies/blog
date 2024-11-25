use std::collections::HashMap;

use crate::{
    engine::Transaction,
    errinput,
    error::Result,
    types::{schema::Table, value::Rows},
};

pub fn insert(
    txn: &impl Transaction,
    table: Table,
    column_map: Option<HashMap<usize, usize>>,
    mut source: Rows,
) -> Result<u64> {
    let mut rows = Vec::new();
    while let Some(values) = source.next().transpose()? {
        if values.len() == table.columns.len() && column_map.is_none() {
            rows.push(values);
            continue;
        }
        if values.len() > table.columns.len() {
            return errinput!("too many values for table:{}", table.name);
        }
        if let Some(column_map) = &column_map {
            if column_map.len() != values.len() {
                return errinput!("column_map length does not match values length");
            }
        }
        let mut row = Vec::with_capacity(table.columns.len());
        for (i, column) in table.columns.iter().enumerate() {
            if let Some(vi) = column_map.as_ref().and_then(|c| c.get(&i)).copied() {
                row.push(values[vi].clone());
            } else if let Some(default) = &column.default {
                row.push(default.clone());
            } else {
                return errinput!("no value given for column {} with no default", column.name);
            }
        }
    }
    let count = rows.len() as u64;
    txn.insert(&table.name, rows)?;
    Ok(count)
}
