use std::{cell::RefCell, collections::HashMap, fmt::Display, hash::Hash, rc::Rc};

use crate::state::Prototype;

pub type Table = HashMap<Value, Value>;

pub type StringRef = Rc<String>;
pub type TableRef = Rc<RefCell<Table>>;
pub type FunctionRef = Rc<Prototype>;
pub type UpvalueRef = Rc<RefCell<Upvalue>>;
pub type ClosureRef = Rc<Closure>;
pub type ArrayRef = Rc<RefCell<Vec<Value>>>;

#[derive(Debug, PartialEq)]
pub enum Upvalue {
    Open { slot: usize },
    Closed { value: Value },
}

#[derive(Debug, PartialEq)]
pub struct Closure {
    pub function: FunctionRef,
    pub upvalues: Vec<UpvalueRef>,
    pub num_upvalues: usize,
}

impl Closure {
    pub fn new(function: FunctionRef) -> Self {
        let upvalues = function.upvalues as usize;
        Self {
            function,
            upvalues: Vec::with_capacity(upvalues),
            num_upvalues: upvalues,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Unit,
    Bool(bool),
    Integer(i64),
    Number(f64),
    String(StringRef),
    Table(TableRef),
    Function(FunctionRef),
    Closure(ClosureRef),
    Array(ArrayRef),
}

impl Value {
    pub fn is_false(&self) -> bool {
        match self {
            Value::Unit | Value::Bool(false) | Value::Integer(0) => true,
            _ => false,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Integer(l0), Self::Integer(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Table(l0), Self::Table(r0)) => l0 == r0,
            (Self::Function(l0), Self::Function(r0)) => l0 == r0,
            (Self::Closure(l0), Self::Closure(r0)) => l0 == r0,
            (Self::Array(l0), Self::Array(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Integer(l), Value::Integer(r)) => l.partial_cmp(r),
            (Value::Number(l), Value::Number(r)) => l.partial_cmp(r),
            _ => None,
        }
    }
}

impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            Value::Unit => {}
            Value::Bool(bool) => bool.hash(state),
            Value::Integer(int) => int.hash(state),
            Value::Number(num) => num.to_le_bytes().hash(state),
            Value::String(str) => str.hash(state),
            Value::Table(table) => Rc::as_ptr(table).hash(state),
            Value::Function(function) => Rc::as_ptr(function).hash(state),
            Value::Closure(closure) => Rc::as_ptr(closure).hash(state),
            Value::Array(array) => Rc::as_ptr(array).hash(state),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Unit => write!(f, "()"),
            Value::Bool(bool) => write!(f, "{bool}"),
            Value::Integer(int) => write!(f, "{int}"),
            Value::Number(num) => write!(f, "{num}"),
            Value::String(str) => write!(f, "{str}"),
            Value::Table(table) => {
                write!(f, "{{")?;
                for (key, value) in &*table.borrow() {
                    write!(f, "{key}:{value},")?;
                }
                write!(f, "}}")?;
                Ok(())
            }
            Value::Function(function) => write!(f, "<fn {}>", function.ident()),
            Value::Closure(closure) => write!(
                f,
                "<fn {}: 0x{:?}>",
                closure.function.ident(),
                Rc::as_ptr(closure)
            ),
            Value::Array(array) => {
                write!(f, "[")?;
                for value in &*array.borrow() {
                    write!(f, "{value},")?;
                }
                write!(f, "]")?;
                Ok(())
            }
        }
    }
}
