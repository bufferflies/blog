use itertools::{Itertools as _, izip};

use crate::{
    errinput,
    error::Result,
    planner::Direction,
    types::{
        expression::{self, Expression},
        value::{Rows, Value},
    },
};

pub fn filter(source: Rows, predicate: Expression) -> Rows {
    Box::new(source.filter_map(move |r| {
        r.and_then(|row| match predicate.evaluate(Some(&row))? {
            Value::Boolean(true) => Ok(Some(row)),
            Value::Boolean(false) | Value::Null => Ok(None),
            value => errinput!("filter returned {value}, expected boolean",),
        })
        .transpose()
    }))
}

pub fn projection(source: Rows, expression: Vec<Expression>) -> Rows {
    Box::new(source.map(move |r| {
        r.and_then(|row| {
            let values: Vec<_> = expression
                .iter()
                .map(|e| e.evaluate(Some(&row)))
                .try_collect()?;
            Ok(values)
        })
    }))
}

pub fn limit(source: Rows, limit: usize) -> Rows {
    Box::new(source.take(limit))
}

pub fn offset(source: Rows, offset: usize) -> Rows {
    Box::new(source.skip(offset))
}

pub fn order(source: Rows, order: Vec<(Expression, Direction)>) -> Result<Rows> {
    let mut irows: Vec<_> = source
        .enumerate()
        .map(|(i, r)| r.map(|row| (i, row)))
        .try_collect()?;
    let mut sort_values = Vec::with_capacity(irows.len());
    for (_, row) in &irows {
        let values: Vec<_> = order
            .iter()
            .map(|(e, _)| e.evaluate(Some(row)))
            .try_collect()?;
        sort_values.push(values)
    }

    irows.sort_by(|&(a, _), &(b, _)| {
        let dirs = order.iter().map(|(_, dir)| dir);
        for (a, b, dir) in izip!(&sort_values[a], &sort_values[b], dirs) {
            match a.cmp(b) {
                std::cmp::Ordering::Equal => {}
                order if *dir == Direction::Descending => return order.reverse(),
                order => return order,
            }
        }
        std::cmp::Ordering::Equal
    });

    Ok(Box::new(irows.into_iter().map(|(_, row)| Ok(row))))
}
