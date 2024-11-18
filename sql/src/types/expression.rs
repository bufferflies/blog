use serde::{Deserialize, Serialize};

use super::value::{Row, Value};
use crate::{errinput, error::Result};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Constant(Value),
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Not(Box<Expression>),

    // /// Compares two expressions for equality：a = b
    Equal(Box<Expression>, Box<Expression>),
    /// Greater than comparison of two values：a > b
    GreaterThan(Box<Expression>, Box<Expression>),
    /// Less than comparison of two values：a < b
    LessThan(Box<Expression>, Box<Expression>),
    /// Adds two expressions：a + b
    Add(Box<Expression>, Box<Expression>),
    /// Subtracts two expressions：a - b
    Subtract(Box<Expression>, Box<Expression>),
    /// Multiplies two expressions：a * b
    Multiply(Box<Expression>, Box<Expression>),
    /// Divides two expressions：a / b
    Divide(Box<Expression>, Box<Expression>),
    /// Takes the remainder of two expressions：a % b
    Remainder(Box<Expression>, Box<Expression>),
    /// Identity of an expression：+a
    Identity(Box<Expression>),
    /// Negates an expression：-a
    Negate(Box<Expression>),
    /// Exponential an expression：a ^ b
    Exponential(Box<Expression>, Box<Expression>),

    SquareRoot(Box<Expression>),

    Like(Box<Expression>, Box<Expression>),
}

impl Expression {
    pub fn evaluate(&self, row: Option<&Row>) -> Result<Value> {
        use Value::*;

        Ok(match self {
            Self::Constant(value) => value.clone(),
            Self::Equal(lhs, rhs) => lhs.evaluate(row)?.checked_eq(&rhs.evaluate(row)?)?,
            Self::GreaterThan(lhs, rhs) => lhs.evaluate(row)?.checked_gt(&rhs.evaluate(row)?)?,
            Self::LessThan(lhs, rhs) => lhs.evaluate(row)?.checked_lt(&rhs.evaluate(row)?)?,
            Self::And(lhs, rhs) => match (lhs.evaluate(row)?, rhs.evaluate(row)?) {
                (Boolean(a), Boolean(b)) => Boolean(a && b),
                (Boolean(b), Null) | (Null, Boolean(b)) if !b => Boolean(false),
                (Boolean(_), Null) | (Null, Boolean(_)) | (Null, Null) => Null,
                (lhs, rhs) => return errinput!("can't AND {lhs} and {rhs}"),
            },
            Self::Or(lhs, rhs) => match (lhs.evaluate(row)?, rhs.evaluate(row)?) {
                (Boolean(a), Boolean(b)) => Boolean(a || b),
                (Boolean(b), Null) | (Null, Boolean(b)) if b => Boolean(true),
                (Boolean(_), Null) | (Null, Boolean(_)) | (Null, Null) => Null,
                (lhs, rhs) => return errinput!("can't OR {lhs} and {rhs}"),
            },
            Self::Not(expr) => match expr.evaluate(row)? {
                Boolean(b) => Boolean(!b),
                Null => Null,
                expr => return errinput!("can't NOT {expr}"),
            },

            Self::Add(lhs, rhs) => lhs.evaluate(row)?.checked_add(&rhs.evaluate(row)?)?,
            #[allow(clippy::float_cmp)]
            Self::Subtract(lhs, rhs) => lhs.evaluate(row)?.checked_sub(&rhs.evaluate(row)?)?,
            Self::Multiply(lhs, rhs) => lhs.evaluate(row)?.checked_mul(&rhs.evaluate(row)?)?,
            Self::Divide(lhs, rhs) => lhs.evaluate(row)?.checked_div(&rhs.evaluate(row)?)?,
            Self::Remainder(lhs, rhs) => lhs.evaluate(row)?.checked_rem(&rhs.evaluate(row)?)?,
            Self::Identity(expr) => match expr.evaluate(row)? {
                v @ (Integer(_) | Float(_) | Null) => v,
                expr => return errinput!("can't take the identity of {expr}"),
            },
            Self::Negate(expr) => expr.evaluate(row)?.checked_neg()?,
            Self::Exponential(lhs, rhs) => lhs.evaluate(row)?.checked_pow(&rhs.evaluate(row)?)?,
            Self::SquareRoot(expr) => match expr.evaluate(row)? {
                Integer(i) => Float((i as f64).sqrt()),
                Float(f) => Float(f.sqrt()),
                _ => return errinput!("can't take square root of {expr:?}"),
            },

            Self::Like(lhs, rhg) => match (lhs.evaluate(row)?, rhg.evaluate(row)?) {
                (String(lhs), String(rhs)) => {
                    // We could precompile the pattern if it's constant, instead
                    // of recompiling it for every row, but this is fine.
                    let pattern = format!(
                        "^{}$",
                        regex::escape(&rhs).replace('%', ".*").replace('_', ".")
                    );
                    Boolean(regex::Regex::new(&pattern)?.is_match(&lhs))
                }
                (String(_), Null) | (Null, String(_)) | (Null, Null) => Null,
                (lhs, rhs) => return errinput!("can't LIKE {lhs} and {rhs}"),
            },
        })
    }
}
