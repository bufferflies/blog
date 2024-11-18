use crate::{
    errinput,
    error::Result,
    parser::ast,
    types::{expression::Expression, value::Value},
};

pub struct Planner {}

impl Planner {
    pub fn build_expression(expr: ast::Expression) -> Result<Expression> {
        use Expression::*;
        let build_fn = |expr: Box<ast::Expression>| -> Result<Box<Expression>> {
            Ok(Box::new(Self::build_expression(*expr)?))
        };
        let ret = match expr {
            ast::Expression::Literal(lit) => Constant(match lit {
                ast::Literal::Null => Value::Null,
                ast::Literal::Boolean(b) => Value::Boolean(b),
                ast::Literal::Integer(i) => Value::Integer(i),
                ast::Literal::Float(f) => Value::Float(f),
                ast::Literal::String(s) => Value::String(s),
            }),
            ast::Expression::Function(function_name, mut expression) => {
                match (function_name.as_str(), expression.len()) {
                    ("sqrt", 1) => SquareRoot(build_fn(Box::new(expression.remove(0)))?),
                    (name, n) => {
                        return errinput!("unknown function {name} with {n} arguments");
                    }
                }
            }
            ast::Expression::Operator(op) => match op {
                ast::Operator::And(lhs, rhs) => And(build_fn(lhs)?, build_fn(rhs)?),
                ast::Operator::Or(lhs, rhs) => Or(build_fn(lhs)?, build_fn(rhs)?),
                ast::Operator::Not(expr) => Not(build_fn(expr)?),

                ast::Operator::Equal(lhs, rhs) => Equal(build_fn(lhs)?, build_fn(rhs)?),
                ast::Operator::NotEqual(lhs, rhs) => {
                    Not(Equal(build_fn(lhs)?, build_fn(rhs)?).into())
                }
                ast::Operator::GreaterThan(lhs, rhs) => GreaterThan(build_fn(lhs)?, build_fn(rhs)?),
                ast::Operator::LessThan(lhs, rhs) => LessThan(build_fn(lhs)?, build_fn(rhs)?),
                ast::Operator::GreaterThanOrEqual(lhs, rhs) => Or(
                    GreaterThan(build_fn(lhs.clone())?, build_fn(rhs.clone())?).into(),
                    Equal(build_fn(lhs)?, build_fn(rhs)?).into(),
                ),
                ast::Operator::LessThanOrEqual(lhs, rhs) => Or(
                    LessThan(build_fn(lhs.clone())?, build_fn(rhs.clone())?).into(),
                    Equal(build_fn(lhs)?, build_fn(rhs)?).into(),
                ),

                ast::Operator::Add(lhs, rhs) => Add(build_fn(lhs)?, build_fn(rhs)?),
                ast::Operator::Subtract(lhs, rhs) => Subtract(build_fn(lhs)?, build_fn(rhs)?),
                ast::Operator::Multiply(lhs, rhs) => Multiply(build_fn(lhs)?, build_fn(rhs)?),
                ast::Operator::Divide(lhs, rhs) => Divide(build_fn(lhs)?, build_fn(rhs)?),
                ast::Operator::Remainder(lhs, rhs) => Remainder(build_fn(lhs)?, build_fn(rhs)?),
                ast::Operator::Exponential(lhs, rhs) => Exponential(build_fn(lhs)?, build_fn(rhs)?),
                ast::Operator::Identity(expr) => Identity(build_fn(expr)?),
                ast::Operator::Negate(expr) => Negate(build_fn(expr)?),

                ast::Operator::Like(lhs, rhs) => Like(build_fn(lhs)?, build_fn(rhs)?),
                op => return errinput!("unsupported operator:{op:?}"),
            },
            e => return errinput!("unsupported expression:{e:?}"),
        };
        Ok(ret)
    }
}
