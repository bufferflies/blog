use crate::types::schema::DataType;

pub type TableName = String;
pub type ColumnName = String;
pub type FunctionName = String;

#[derive(Debug)]
pub enum Statement {
    Explain(Box<Statement>),
    Select {
        select: Vec<(Expression, Option<String>)>,
        from: Vec<TableName>,
        r#where: Option<Expression>,
        limit: Option<Expression>,
    },
    CrateTable {
        table_name: TableName,
        columns: Vec<Column>,
    },
    DropTable {
        table_name: TableName,
        if_exists: bool,
    },
    Insert {
        table_name: TableName,
        columns: Option<Vec<ColumnName>>,
        values: Vec<Vec<Expression>>,
    },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Expression {
    All,
    Column(Option<TableName>, ColumnName),
    Literal(Literal),
    Operator(Operator),
    /// A function call (name and parameters).
    Function(FunctionName, Vec<Expression>),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
}

impl std::cmp::Eq for Literal {}
impl std::hash::Hash for Literal {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Literal::Null => {}
            Literal::Boolean(b) => b.hash(state),
            Literal::Integer(i) => i.hash(state),
            Literal::Float(f) => f.to_bits().hash(state),
            Literal::String(s) => s.hash(state),
        }
    }
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Literal::Null, Literal::Null) => true,
            (Literal::Boolean(a), Literal::Boolean(b)) => a == b,
            (Literal::Integer(a), Literal::Integer(b)) => a == b,
            (Literal::Float(a), Literal::Float(b)) => a == b,
            (Literal::String(a), Literal::String(b)) => a == b,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Operator {
    And(Box<Expression>, Box<Expression>), // a AND b
    Or(Box<Expression>, Box<Expression>),  // a OR b
    Not(Box<Expression>),                  // NOT a

    Equal(Box<Expression>, Box<Expression>),       // a = b
    NotEqual(Box<Expression>, Box<Expression>),    // a != b
    GreaterThan(Box<Expression>, Box<Expression>), // a > b
    LessThan(Box<Expression>, Box<Expression>),    // a < b
    GreaterThanOrEqual(Box<Expression>, Box<Expression>), // a >= b
    LessThanOrEqual(Box<Expression>, Box<Expression>), // a <= b
    Is(Box<Expression>, Literal),                  // a IS NULL

    Add(Box<Expression>, Box<Expression>),         // a + b
    Subtract(Box<Expression>, Box<Expression>),    // a - b
    Multiply(Box<Expression>, Box<Expression>),    // a * b
    Divide(Box<Expression>, Box<Expression>),      // a / b
    Remainder(Box<Expression>, Box<Expression>),   // a % b
    Negate(Box<Expression>),                       // -a
    Identity(Box<Expression>),                     // +a
    Exponential(Box<Expression>, Box<Expression>), // a ^ b

    Like(Box<Expression>, Box<Expression>), // a LIKE b
}

impl From<Operator> for Expression {
    fn from(op: Operator) -> Self {
        Self::Operator(op)
    }
}

impl From<Literal> for Expression {
    fn from(literal: Literal) -> Self {
        Self::Literal(literal)
    }
}

impl Expression {
    pub fn walk(&self, visitor: &mut impl FnMut(&Expression) -> bool) -> bool {
        use Operator::*;
        if !visitor(self) {
            return false;
        }
        match self {
            Self::Operator(And(lhs, rhs))
            | Self::Operator(Or(lhs, rhs))
            | Self::Operator(Equal(lhs, rhs))
            | Self::Operator(NotEqual(lhs, rhs))
            | Self::Operator(GreaterThan(lhs, rhs))
            | Self::Operator(LessThan(lhs, rhs))
            | Self::Operator(GreaterThanOrEqual(lhs, rhs))
            | Self::Operator(LessThanOrEqual(lhs, rhs))
            | Self::Operator(Add(lhs, rhs))
            | Self::Operator(Subtract(lhs, rhs))
            | Self::Operator(Multiply(lhs, rhs))
            | Self::Operator(Divide(lhs, rhs))
            | Self::Operator(Remainder(lhs, rhs))
            | Self::Operator(Exponential(lhs, rhs))
            | Self::Operator(Like(lhs, rhs)) => lhs.walk(visitor) && rhs.walk(visitor),
            Self::Operator(Not(expr))
            | Self::Operator(Negate(expr))
            | Self::Operator(Is(expr, _))
            | Self::Operator(Identity(expr)) => expr.walk(visitor),
            Self::Function(_, exprs) => exprs.iter().any(|expr| expr.walk(visitor)),
            Self::All | Self::Column(..) | Self::Literal(_) => true,
        }
    }
}

/// A CREATE TABLE column definition.
#[derive(Debug)]
pub struct Column {
    pub name: String,
    pub datatype: DataType,
    pub primary_key: bool,
    pub nullable: Option<bool>,
    pub default: Option<Expression>,
    pub unique: bool,
}
