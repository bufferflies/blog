use super::session::Session;
use crate::{
    errinput,
    error::Result,
    types::{
        expression::Expression,
        schema::Table,
        value::{Row, Rows, Value},
    },
};

pub trait Engine<'a>: Sized {
    type Transaction: Transaction + Catalog + 'a;

    fn begin(&'a self) -> Result<Self::Transaction>;
    fn begin_read_only(&'a self) -> Result<Self::Transaction>;

    fn session(&'a self) -> Session<'a, Self> {
        Session::new(self)
    }
}

pub trait Transaction {
    fn commit(self) -> Result<()>;
    fn rollback(self) -> Result<()>;
    fn insert(&self, table: &str, rows: Vec<Row>) -> Result<()>;
    fn get(&self, table: &str, ids: &[Value]) -> Result<Vec<Row>>;
    fn scan(&self, table: &str, filter: Option<Expression>) -> Result<Rows>;
}

pub trait Catalog {
    fn create_table(&self, table: Table) -> Result<()>;
    fn get_table(&self, table: &str) -> Result<Option<Table>>;
    fn list_tables(&self) -> Result<Vec<Table>>;

    fn must_get_table(&self, table: &str) -> Result<Table> {
        self.get_table(table)?
            .ok_or_else(|| errinput!("table {table} does not exist"))
    }
}
