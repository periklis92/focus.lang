#[derive(Debug, PartialEq)]
pub enum Statement {
    Let {
        ident: String,
        value: Option<Expression>,
    },
    Function {
        ident: String,
        args: Vec<String>,
        expr: Expression,
    },
    Import {
        source: ImportSource,
        imports: Vec<Import>,
    },
    Expression(Expression),
}

impl Statement {
    pub fn is_expression(&self) -> bool {
        matches!(self, Statement::Expression(_))
    }
}

#[derive(Debug, PartialEq)]
pub enum Expression {
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
    Table(Vec<TableEntry>),
    Literal(Literal),
    Block(Vec<Statement>),
    Path {
        ident: String,
        parts: Vec<PathPart>,
    },
    Call {
        callee: Box<Expression>,
        args: Vec<Expression>,
    },
    Function {
        args: Vec<String>,
        expr: Box<Expression>,
    },
    If {
        condition: Box<Expression>,
        block: Box<Expression>,
        r#else: Option<Box<Expression>>,
    },
    InterpolatedString {
        format: String,
        arguments: Vec<InterpolatedArgument>,
    },
}

#[derive(Debug, PartialEq)]
pub struct TableEntry {
    pub key: Expression,
    pub value: Expression,
}

#[derive(Debug, PartialEq)]
pub enum PathPart {
    Ident(String),
    Index(Expression),
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
pub enum Literal {
    Unit,
    Bool(bool),
    Integer(i64),
    Number(f64),
    String(String),
}

#[derive(Debug, PartialEq)]
pub struct InterpolatedArgument {
    pub offset: usize,
    pub expression: Expression,
}

#[derive(Debug, PartialEq)]
pub enum ImportSource {
    Module(String),
    File(String),
}

#[derive(Debug, PartialEq)]
pub enum Import {
    Local { ident: String, alias: String },
    All { alias: Option<String> },
}
