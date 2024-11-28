use super::Node;
use crate::{
    error::Result,
    types::{expression::Expression, value::Value},
};

pub type Optimizer = fn(Node) -> Result<Node>;

pub static OPTIMIZERS: &[(&str, Optimizer)] = &[
    ("Constant folding", fold_constants),
    ("Filter pushdown", push_filter),
    ("Short circuit", short_circuit),
];

pub fn short_circuit(node: Node) -> Result<Node> {
    use Expression::*;
    use Value::*;
    fn nothing(node: &Node) -> Node {
        let columns = (0..node.columns()).map(|i| node.column_label(i)).collect();
        Node::Nothing { columns }
    }

    let xform = |node| {
        match node {
            Node::Filter {
                source,
                predicate: Constant(Boolean(true)),
            } => *source,

            // select * from t where true
            Node::Scan {
                table,
                filter: Some(Constant(Boolean(true))),
            } => Node::Scan {
                table,
                filter: None,
            },
            ref node @ Node::Filter {
                predicate: Constant(Boolean(false) | Null),
                ..
            } => nothing(node),
            // select * from t limit 0
            ref node @ Node::Limit { limit: 0, .. } => nothing(node),
            // select * from t where false
            ref node @ Node::Scan {
                filter: Some(Constant(Boolean(false) | Null)),
                ..
            } => nothing(node),
            node => node,
        }
    };
    node.transform(&Ok, &|node| Ok(xform(node)))
}

pub fn fold_constants(node: Node) -> Result<Node> {
    use Expression::*;
    use Value::*;

    let xform = |mut expr: Expression| {
        if !expr.contains(&|expr| matches!(expr, Column(_))) {
            return expr.evaluate(None).map(Constant);
        }
        expr = match expr {
            And(lhs, rhs) => match (*lhs, *rhs) {
                (Constant(Boolean(false)), _) | (_, Constant(Boolean(false))) => {
                    Constant(Boolean(false))
                }
                (Constant(Boolean(true)), expr) | (expr, Constant(Boolean(true))) => expr,
                (lhs, rhs) => And(lhs.into(), rhs.into()),
            },
            Or(lhs, rhs) => match (*lhs, *rhs) {
                (Constant(Boolean(true)), _) | (_, Constant(Boolean(true))) => {
                    Constant(Boolean(true))
                }
                (Constant(Boolean(false)), expr) | (expr, Constant(Boolean(false))) => expr,
                (lhs, rhs) => Or(lhs.into(), rhs.into()),
            },
            _ => expr,
        };
        Ok(expr)
    };
    node.transform(&|node| node.transform_expressions(&Ok, &xform), &Ok)
}

pub fn push_filter(node: Node) -> Result<Node> {
    fn push_into(expr: Expression, target: &mut Node) -> Option<Expression> {
        match target {
            Node::Scan { filter, .. } => {
                *filter = match filter.take() {
                    Some(filter) => Some(Expression::And(expr.into(), filter.into())),
                    None => Some(expr),
                }
            }
            _ => return Some(expr),
        }
        None
    }

    fn push_filter(node: Node) -> Node {
        let Node::Filter {
            mut source,
            predicate,
        } = node
        else {
            return node;
        };
        if let Some(predicate) = push_into(predicate, &mut source) {
            return Node::Filter { source, predicate };
        }
        xform(*source)
    }
    fn push_join(node: Node) -> Node {
        node
    }
    fn xform(node: Node) -> Node {
        push_join(push_filter(node))
    }

    node.transform(&|node| Ok(xform(node)), &Ok)
}
