use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use super::Catalog;
use crate::{
    encoding::{self, Key as _, Value as _},
    errinput,
    error::Result,
    storage::{self, mvcc},
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
        todo!()
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
        todo!()
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
            self.txn.set(
                &Key::Row((&table.name).into(), id.into()).encode(),
                row.encode(),
            )?;

            // todd: update secondary indexes
        }
        Ok(())
    }

    fn get(&self, table: &str, ids: &[Value]) -> Result<Vec<Row>> {
        ids.iter()
            .filter_map(|id| self.get_row(table, &id.normalize_ref()).transpose())
            .collect()
    }

    fn scan(&self, _table: &str, _filter: Option<Expression>) -> Result<Rows> {
        todo!()
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

impl<'a> encoding::Value for Key<'a> {}
