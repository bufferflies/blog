use std::fmt::write;

use serde::{Deserialize, Serialize};

use super::value::Value;
use crate::encoding;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Table {
    pub name: String,
    pub primary_key: usize,
    pub columns: Vec<Column>,
}

impl encoding::Value for Table {}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Column {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
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

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Boolean => write!(f, "BOOLEAN"),
            Self::Integer => write!(f, "INTEGER"),
            Self::Float => write!(f, "FLOAT"),
            Self::String => write!(f, "STRING"),
        }
    }
}
