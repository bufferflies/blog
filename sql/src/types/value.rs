use serde::{Deserialize, Serialize};

use crate::{errinput, error::Result};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            _ => false,
        }
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
