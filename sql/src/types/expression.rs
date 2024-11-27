use serde::{Deserialize, Serialize};

use super::value::{Row, Value};
use crate::{errinput, error::Result, planner::Node, types::value::Label};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Constant(Value),
    Column(usize),

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

    Is(Box<Expression>, Value),
}

impl Expression {
    pub fn evaluate(&self, row: Option<&Row>) -> Result<Value> {
        use Value::*;

        Ok(match self {
            Self::Constant(value) => value.clone(),
            Self::Column(index) => match row {
                Some(row) => row.get(*index).cloned().expect("short row").clone(),
                None => {
                    return errinput!("can't reference column {index} with constant evaluation");
                }
            },
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

            Self::Is(expr, Null) => Boolean(expr.evaluate(row)? == Null),
            Self::Is(expr, Float(f)) if f.is_nan() => match expr.evaluate(row)? {
                Float(f) => Boolean(f.is_nan()),
                Null => Null,
                v => return errinput!("IS NAN can't be used with {}", v.data_type().unwrap()),
            },
            Self::Is(_, v) => panic!("invalid IS value {v}"),
        })
    }

    pub fn format(&self, node: &Node) -> String {
        use Expression::*;

        fn precedence(expr: &Expression) -> u8 {
            match expr {
                Column(_) | Constant(_) | SquareRoot(_) => 11,
                Identity(_) | Negate(_) => 10,
                &Exponential(..) => 8,
                Multiply(..) | Divide(..) | Remainder(..) => 7,
                Add(..) | Subtract(..) => 6,
                GreaterThan(..) | LessThan(..) => 5,
                Equal(..) | Like(..) | Is(..) => 4,
                Not(_) => 3,
                And(..) => 2,
                Or(..) => 1,
            }
        }

        let format = |expr: &Expression| {
            let mut string = expr.format(node);
            if precedence(expr) < precedence(self) {
                string = format!("({string})");
            }
            string
        };

        match self {
            Constant(value) => format!("{value}"),
            Column(index) => match node.column_label(*index) {
                Label::None => format!("#{index}"),
                label => format!("{label}"),
            },

            And(lhs, rhs) => format!("{} AND {}", format(lhs), format(rhs)),
            Or(lhs, rhs) => format!("{} OR {}", format(lhs), format(rhs)),
            Not(expr) => format!("NOT {}", format(expr)),

            Equal(lhs, rhs) => format!("{} = {}", format(lhs), format(rhs)),
            GreaterThan(lhs, rhs) => format!("{} > {}", format(lhs), format(rhs)),
            LessThan(lhs, rhs) => format!("{} < {}", format(lhs), format(rhs)),
            Is(expr, Value::Null) => format!("{} IS NULL", format(expr)),
            Is(expr, Value::Float(f)) if f.is_nan() => format!("{} IS NAN", format(expr)),
            Is(_, v) => panic!("unexpected IS value {v}"),

            Add(lhs, rhs) => format!("{} + {}", format(lhs), format(rhs)),
            Divide(lhs, rhs) => format!("{} / {}", format(lhs), format(rhs)),
            Exponential(lhs, rhs) => format!("{} ^ {}", format(lhs), format(rhs)),
            Identity(expr) => format(expr),
            Multiply(lhs, rhs) => format!("{} * {}", format(lhs), format(rhs)),
            Negate(expr) => format!("-{}", format(expr)),
            Remainder(lhs, rhs) => format!("{} % {}", format(lhs), format(rhs)),
            SquareRoot(expr) => format!("sqrt({})", format(expr)),
            Subtract(lhs, rhs) => format!("{} - {}", format(lhs), format(rhs)),

            Like(lhs, rhs) => format!("{} LIKE {}", format(lhs), format(rhs)),
        }
    }
}
