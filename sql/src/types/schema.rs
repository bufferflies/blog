use serde::{Deserialize, Serialize};

use super::value::Value;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Table {
    pub name: String,
    pub primary_key: usize,
    pub columns: Vec<Column>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Column {
    pub name: String,
    pub data_type: DataType,
    pub primary_key: bool,
    pub nullable: Option<bool>,
    pub default: Option<Value>,
    pub unique: bool,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum DataType {
    Boolean,
    Integer,
    Float,
    String,
}
