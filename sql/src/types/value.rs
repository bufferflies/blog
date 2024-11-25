use std::borrow::Cow;

use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};

use crate::{
    encoding, errdata, errinput,
    error::{Error, Result},
    parser::ast::{self, TableName},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
}

impl encoding::Value for Value {}

impl std::cmp::PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Boolean(l), Self::Boolean(r)) => l == r,
            (Self::Integer(l), Self::Integer(r)) => l == r,
            (Self::Float(l), Self::Float(r)) => l == r || l.is_nan() && r.is_nan(),
            (Self::String(l), Self::String(r)) => l == r,
            (l, r) => core::mem::discriminant(l) == core::mem::discriminant(r),
        }
    }
}
impl std::cmp::Eq for Value {}

// For ordering purposes, we consider NULL and NaN equal. We establish a total
// order across all types, even though mixed types will rarely/never come up.
impl Ord for Value {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering::*;
        use Value::*;
        match (self, other) {
            (Null, Null) => Equal,
            (Boolean(a), Boolean(b)) => a.cmp(b),
            (Integer(a), Integer(b)) => a.cmp(b),
            (Integer(a), Float(b)) => (*a as f64).total_cmp(b),
            (Float(a), Integer(b)) => a.total_cmp(&(*b as f64)),
            (Float(a), Float(b)) => a.total_cmp(b),
            (String(a), String(b)) => a.cmp(b),

            (Null, _) => Less,
            (_, Null) => Greater,
            (Boolean(_), _) => Less,
            (_, Boolean(_)) => Greater,
            (Float(_), _) => Less,
            (_, Float(_)) => Greater,
            (Integer(_), _) => Less,
            (_, Integer(_)) => Greater,
            // String is ordered last.
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Value {
    pub fn normalize(&mut self) {
        if let Cow::Owned(normalized) = self.normalize_ref() {
            *self = normalized;
        }
    }

    pub fn normalize_ref(&self) -> Cow<'_, Self> {
        if let Self::Float(f) = self {
            if (f.is_nan() || *f == -0.0) && f.is_sign_negative() {
                return Cow::Owned(Self::Float(-f));
            }
        }
        Cow::Borrowed(self)
    }

    // Returns true if the value is normalized.
    pub fn is_normalized(&self) -> bool {
        matches!(self.normalize_ref(), Cow::Borrowed(_))
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Null => f.write_str("NULL"),
            Self::Boolean(true) => f.write_str("TRUE"),
            Self::Boolean(false) => f.write_str("FALSE"),
            Self::Integer(integer) => integer.fmt(f),
            Self::Float(float) => write!(f, "{float:?}"),
            Self::String(string) => write!(f, "'{}'", string.escape_debug()),
        }
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Boolean(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Float(v)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Integer(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(v.to_owned())
    }
}

impl TryFrom<Value> for bool {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        let Value::Boolean(b) = value else {
            return errdata!("not a boolean: {value}");
        };
        Ok(b)
    }
}

impl TryFrom<Value> for f64 {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        let Value::Float(f) = value else {
            return errdata!("not a float: {value}");
        };
        Ok(f)
    }
}

impl TryFrom<Value> for i64 {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        let Value::Integer(i) = value else {
            return errdata!("not an integer: {value}");
        };
        Ok(i)
    }
}

impl TryFrom<Value> for String {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        let Value::String(s) = value else {
            return errdata!("not a string: {value}");
        };
        Ok(s)
    }
}

impl<'a> From<&'a Value> for Cow<'a, Value> {
    fn from(v: &'a Value) -> Self {
        Cow::Borrowed(v)
    }
}

impl Value {
    pub fn checked_add(&self, other: &Self) -> Result<Self> {
        use Value::*;
        Ok(match (self, other) {
            (Integer(lhs), Integer(rhs)) => match lhs.checked_add(*rhs) {
                Some(i) => Integer(i),
                None => return errinput!("integer overflow"),
            },
            (Integer(lhs), Float(rhs)) => Float(*lhs as f64 + rhs),
            (Float(lhs), Integer(rhs)) => Float(lhs + *rhs as f64),
            (Float(lhs), Float(rhs)) => Float(lhs + rhs),
            (Null, Integer(_) | Float(_) | Null) => Null,
            (Integer(_) | Float(_), Null) => Null,
            (lhs, rhs) => return errinput!("can't add {lhs} and {rhs}"),
        })
    }

    pub fn checked_sub(&self, other: &Self) -> Result<Self> {
        use Value::*;
        Ok(match (self, other) {
            (Integer(lhs), Integer(rhs)) => match lhs.checked_sub(*rhs) {
                Some(i) => Integer(i),
                None => return errinput!("integer overflow"),
            },
            (Integer(lhs), Float(rhs)) => Float(*lhs as f64 - rhs),
            (Float(lhs), Integer(rhs)) => Float(lhs - *rhs as f64),
            (Float(lhs), Float(rhs)) => Float(lhs - rhs),
            (Null, Integer(_) | Float(_) | Null) => Null,
            (Integer(_) | Float(_), Null) => Null,
            (lhs, rhs) => return errinput!("can't subtract {lhs} and {rhs}"),
        })
    }

    pub fn checked_mul(&self, other: &Self) -> Result<Self> {
        use Value::*;
        Ok(match (self, other) {
            (Integer(lhs), Integer(rhs)) => match lhs.checked_mul(*rhs) {
                Some(i) => Integer(i),
                None => return errinput!("integer overflow"),
            },
            (Integer(lhs), Float(rhs)) => Float(*lhs as f64 * rhs),
            (Float(lhs), Integer(rhs)) => Float(lhs * (*rhs as f64)),
            (Float(lhs), Float(rhs)) => Float(lhs * rhs),
            (Null, Integer(_) | Float(_) | Null) => Null,
            (Integer(_) | Float(_), Null) => Null,
            (lhs, rhs) => return errinput!("can't multiply {lhs} and {rhs}"),
        })
    }

    pub fn checked_div(&self, other: &Self) -> Result<Self> {
        use Value::*;
        Ok(match (self, other) {
            (Integer(_), Integer(0) | Float(0.0)) => return errinput!("can't divide by zero"),
            (Integer(lhs), Integer(rhs)) => match lhs.checked_div(*rhs) {
                Some(i) => Integer(i),
                None => return errinput!("integer overflow"),
            },
            (Integer(lhs), Float(rhs)) => Float(*lhs as f64 / rhs),
            (Float(lhs), Integer(rhs)) => Float(lhs / (*rhs as f64)),
            (Float(lhs), Float(rhs)) => Float(lhs / rhs),
            (Null, Integer(_) | Float(_) | Null) => Null,
            (Integer(_) | Float(_), Null) => Null,
            (lhs, rhs) => return errinput!("can't divide {lhs} and {rhs}"),
        })
    }

    pub fn checked_rem(&self, other: &Self) -> Result<Self> {
        use Value::*;
        Ok(match (self, other) {
            (Integer(_), Integer(0) | Float(0.0)) => return errinput!("can't divide by zero"),
            (Integer(lhs), Integer(rhs)) => match lhs.checked_rem(*rhs) {
                Some(i) => Integer(i),
                None => return errinput!("integer overflow"),
            },
            (Integer(lhs), Float(rhs)) => Float(*lhs as f64 % rhs),
            (Float(lhs), Integer(rhs)) => Float(lhs % (*rhs as f64)),
            (Float(lhs), Float(rhs)) => Float(lhs % rhs),
            (Null, Integer(_) | Float(_) | Null) => Null,
            (Integer(_) | Float(_), Null) => Null,
            (lhs, rhs) => return errinput!("can't take remainder of {lhs} and {rhs}"),
        })
    }

    pub fn checked_neg(&self) -> Result<Self> {
        use Value::*;
        Ok(match self {
            Integer(i) => Integer(-i),
            Float(f) => Float(-f),
            Null => Null,
            _ => return errinput!("can't negate {self}"),
        })
    }

    pub fn checked_eq(&self, other: &Self) -> Result<Self> {
        use Value::*;
        Ok(match (self, other) {
            (Boolean(lhs), Boolean(rhs)) => Boolean(lhs == rhs),
            (Integer(lhs), Integer(rhs)) => Boolean(lhs == rhs),
            (Integer(lhs), Float(rhs)) => Boolean(*lhs as f64 == *rhs),
            (Float(lhs), Integer(rhs)) => Boolean(*lhs == *rhs as f64),
            (Float(lhs), Float(rhs)) => Boolean(lhs == rhs),
            (String(lhs), String(rhs)) => Boolean(lhs == rhs),
            (Null, _) | (_, Null) => Null,
            (lhs, rhs) => return errinput!("can't compare {lhs} and {rhs}"),
        })
    }

    pub fn checked_gt(&self, other: &Self) -> Result<Self> {
        use Value::*;
        Ok(match (self, other) {
            #[allow(clippy::bool_comparison)]
            (Boolean(lhs), Boolean(rhs)) => Boolean(lhs > rhs),
            (Integer(lhs), Integer(rhs)) => Boolean(lhs > rhs),
            (Integer(lhs), Float(rhs)) => Boolean(*lhs as f64 > *rhs),
            (Float(lhs), Integer(rhs)) => Boolean(*lhs > *rhs as f64),
            (Float(lhs), Float(rhs)) => Boolean(lhs > rhs),
            (String(lhs), String(rhs)) => Boolean(lhs > rhs),
            (Null, _) | (_, Null) => Null,
            (lhs, rhs) => return errinput!("can't compare {lhs} and {rhs}"),
        })
    }

    pub fn checked_lt(&self, other: &Self) -> Result<Self> {
        use Value::*;
        Ok(match (self, other) {
            #[allow(clippy::bool_comparison)]
            (Boolean(lhs), Boolean(rhs)) => Boolean(lhs < rhs),
            (Integer(lhs), Integer(rhs)) => Boolean(lhs < rhs),
            (Integer(lhs), Float(rhs)) => Boolean((*lhs as f64) < *rhs),
            (Float(lhs), Integer(rhs)) => Boolean(*lhs < *rhs as f64),
            (Float(lhs), Float(rhs)) => Boolean(lhs < rhs),
            (String(lhs), String(rhs)) => Boolean(lhs < rhs),
            (Null, _) | (_, Null) => Null,
            (lhs, rhs) => return errinput!("can't compare {lhs} and {rhs}"),
        })
    }

    pub fn checked_pow(&self, other: &Self) -> Result<Self> {
        use Value::*;
        Ok(match (self, other) {
            (Integer(lhs), Integer(rhs)) if *rhs >= 0 => {
                let rhs = (*rhs)
                    .try_into()
                    .or_else(|_| errinput!("integer overflow"))?;
                match lhs.checked_pow(rhs) {
                    Some(i) => Integer(i),
                    None => return errinput!("integer overflow"),
                }
            }
            (Integer(lhs), Integer(rhs)) => Float((*lhs as f64).powf(*rhs as f64)),
            (Integer(lhs), Float(rhs)) => Float((*lhs as f64).powf(*rhs)),
            (Float(lhs), Integer(rhs)) => Float((lhs).powi(*rhs as i32)),
            (Float(lhs), Float(rhs)) => Float((lhs).powf(*rhs)),
            (Integer(_) | Float(_), Null) => Null,
            (Null, Integer(_) | Float(_) | Null) => Null,
            (lhs, rhs) => return errinput!("can't Exponential {lhs} and {rhs}"),
        })
    }
}

pub type Row = Vec<Value>;

/// A row iterator.
pub type Rows = Box<dyn RowIterator>;

pub trait RowIterator: Iterator<Item = Result<Row>> + DynClone {}
impl<I: Iterator<Item = Result<Row>> + DynClone> RowIterator for I {}
dyn_clone::clone_trait_object!(RowIterator);

#[derive(Debug,Clone,PartialEq,Serialize, Deserialize)]
pub enum Label {
    /// no label
    None,
    /// an unqualified column name
    Unqualified(String),
    /// a fully qualified table/column name
    Qualified(TableName, String),
}

impl From<Option<String>> for Label {
    fn from(label_name: Option<String>) -> Self {
        label_name.map(Label::Unqualified).unwrap_or(Label::None)
    }
}

impl From<Label> for ast::Expression {
    fn from(value: Label) -> Self {
        match value {
            Label::None => panic!("can't convert none label to ast expression"),
            Label::Unqualified(name) => ast::Expression::Column(None, name),
            Label::Qualified(table_name, name) => ast::Expression::Column(Some(table_name), name),
        }
    }
}
