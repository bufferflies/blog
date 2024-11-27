use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use super::Catalog;
use crate::{
    encoding::{self, Key as _, Value as _, keycode},
    errinput,
    error::Result,
    storage::{
        self,
        engine::ScanIterator,
        mvcc::{self},
    },
    types::{
        expression::Expression,
        schema::Table,
        value::{Row, Rows, Value},
    },
};

pub struct Local<E: storage::Engine + 'static> {
    pub mvcc: storage::MVCC<E>,
}

impl<E: storage::Engine> Local<E> {
    pub fn new(engine: E) -> Self {
        Self {
            mvcc: storage::MVCC::new(engine),
        }
    }
}

impl<'a, E: storage::Engine> super::Engine<'a> for Local<E> {
    type Transaction = Transaction<E>;

    fn begin(&'a self) -> Result<Self::Transaction> {
        Ok(Self::Transaction::new(self.mvcc.begin()?))
    }

    fn begin_read_only(&'a self) -> Result<Self::Transaction> {
        Ok(Self::Transaction::new(self.mvcc.begin_read_only()?))
    }
}

pub struct Transaction<E: storage::Engine + 'static> {
    txn: mvcc::TransactionInner<E>,
}

impl<E: storage::Engine> Catalog for Transaction<E> {
    fn create_table(&self, table: Table) -> Result<()> {
        log::info!("create table {:?}", table);
        if self.get_table(&table.name)?.is_some() {
            return errinput!("table {} already exists", table.name);
        }
        self.txn
            .set(&Key::Table((&table.name).into()).encode(), table.encode())
    }

    fn get_table(&self, table: &str) -> Result<Option<Table>> {
        self.txn
            .get(&Key::Table(table.into()).encode())?
            .map(|v| Table::decode(&v))
            .transpose()
    }

    fn list_tables(&self) -> Result<Vec<Table>> {
        errinput!("list_tables not implemented")
    }
}

impl<E: storage::Engine> Transaction<E> {
    pub fn new(txn: mvcc::TransactionInner<E>) -> Self {
        Self { txn }
    }

    /// Fetch a single row by primary key, or not if it doesn't exist.
    /// the key must already be normalized.
    fn get_row(&self, _table: &str, id: &Value) -> Result<Option<Row>> {
        debug_assert!(id.is_normalized(), "value not normalized");
        errinput!("get_row not implemented")
    }
}

impl<E: storage::Engine> super::Transaction for Transaction<E> {
    fn commit(self) -> Result<()> {
        self.txn.commit()
    }

    fn rollback(self) -> Result<()> {
        self.txn.rollback()
    }

    fn insert(&self, table_name: &str, rows: Vec<Row>) -> Result<()> {
        let table = self.must_get_table(table_name)?;
        for mut row in rows {
            row.iter_mut().for_each(|v| v.normalize());
            let id = &row[table.primary_key];
            let key = Key::Row((&table.name).into(), id.into()).encode();
            self.txn.set(&key, row.encode())?;

            // todo: update secondary indexes
        }
        Ok(())
    }

    fn get(&self, table: &str, ids: &[Value]) -> Result<Vec<Row>> {
        ids.iter()
            .filter_map(|id| self.get_row(table, &id.normalize_ref()).transpose())
            .collect()
    }

    fn scan(&self, table: &str, filter: Option<Expression>) -> Result<Rows> {
        let key = KeyPrefix::Row(table.into()).encode();
        let rows = self
            .txn
            .scan_prefix(&key)
            .map(|result| result.and_then(|(_, value)| Row::decode(&value)));
        let Some(filter) = filter else {
            return Ok(Box::new(rows));
        };
        let rows = rows.filter_map(move |result| {
            result
                .and_then(|row| match filter.evaluate(Some(&row))? {
                    Value::Boolean(true) => Ok(Some(row)),
                    Value::Boolean(false) | Value::Null => Ok(None),
                    value => errinput!("filter returned {value}, expected boolean"),
                })
                .transpose()
        });
        Ok(Box::new(rows))
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Key<'a> {
    /// A table schema by table name.
    Table(Cow<'a, str>),
    /// An index entry, by table name, index name, and index value.
    Index(Cow<'a, str>, Cow<'a, str>, Cow<'a, Value>),
    /// A table row, by table name and primary key value.
    Row(Cow<'a, str>, Cow<'a, Value>),
}

impl<'a> encoding::Key<'a> for Key<'a> {}

/// Key prefixes, allowing prefix scans of specific parts of the keyspace. These
/// must match the keys -- in particular, the enum variant indexes must match.
#[derive(Deserialize, Serialize)]
enum KeyPrefix<'a> {
    /// All table schemas.
    Table,
    /// An entire table index, by table and index name.
    Index(Cow<'a, str>, Cow<'a, str>),
    /// An entire table's rows, by table name.
    Row(Cow<'a, str>),
}

impl<'a> encoding::Key<'a> for KeyPrefix<'a> {}
