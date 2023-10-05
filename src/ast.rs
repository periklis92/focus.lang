use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
    io::Write,
};

#[derive(Debug, PartialEq)]
pub enum Expression {
    Local {
        ident: String,
        value: Option<Box<Expression>>,
    },
    UnaryOperation {
        operand: Box<Expression>,
        operation: UnaryOperation,
    },
    Operation {
        lhs: Box<Expression>,
        operation: Operation,
        rhs: Box<Expression>,
    },
    Array(Vec<Expression>),
    Table(BTreeMap<PathPart, Expression>),
    Literal(Literal),
    Block(Vec<Expression>),
    Path {
        expr: Box<Expression>,
        parts: Vec<PathPart>,
    },
    Call {
        path: Box<Expression>,
        args: Vec<Expression>,
    },
    Function {
        ident: Option<String>,
        args: Vec<String>,
        expr: Box<Expression>,
    },
    Ident(String),
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum PathPart {
    Ident(String),
    Number(i64),
    String(String),
}

#[derive(Debug, PartialEq)]
pub enum UnaryOperation {
    Not,
    Negate,
}

#[derive(Debug, PartialEq)]
pub enum Operation {
    Assignment,
    Arithmetic(ArithmeticOperator),
    Comparison(ComparisonOperator),
    Boolean(BooleanOperator),
    Binary(BinaryOperator),
}

impl Operation {
    pub fn precedence(&self) -> i32 {
        match self {
            Operation::Assignment => 10,
            Operation::Comparison(_) => 20,
            Operation::Boolean(_) => 20,
            Operation::Arithmetic(ArithmeticOperator::Add | ArithmeticOperator::Subtract) => 30,
            Operation::Arithmetic(
                ArithmeticOperator::Multiply
                | ArithmeticOperator::Divide
                | ArithmeticOperator::IDivide
                | ArithmeticOperator::Modulus,
            ) => 40,
            Operation::Binary(_) => 50,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ArithmeticOperator {
    Add,
    Subtract,
    Divide,
    IDivide,
    Multiply,
    Modulus,
}

#[derive(Debug, PartialEq)]
pub enum ComparisonOperator {
    Less,
    LessEqual,
    Equal,
    NotEqual,
    GreaterEqual,
    Greater,
}

#[derive(Debug, PartialEq)]
pub enum BooleanOperator {
    And,
    Or,
}

#[derive(Debug, PartialEq)]
pub enum BinaryOperator {
    And,
    Or,
    Xor,
    Lsh,
    Rsh,
    Not,
}

#[derive(Debug, PartialEq)]
pub enum Literal {
    Unit,
    Bool(bool),
    Integer(i64),
    Number(f64),
    String(String),
}
