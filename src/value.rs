use std::{cell::RefCell, collections::HashMap, fmt::Display, hash::Hash, rc::Rc};

use crate::{
    state::{Module, Prototype},
    vm::Vm,
};

pub type Table = HashMap<Value, Value>;

pub type StringRef = Rc<String>;
pub type TableRef = Rc<RefCell<Table>>;
pub type PrototypeRef = Rc<Prototype>;
pub type FunctionRef = Rc<NativeFunction>;
pub type UpvalueRef = Rc<RefCell<Upvalue>>;
pub type ClosureRef = Rc<Closure>;
pub type ArrayRef = Rc<RefCell<Vec<Value>>>;
pub type ModuleRef = Rc<Module>;

#[derive(Debug, PartialEq)]
pub enum Upvalue {
    Open { slot: usize },
    Closed { value: Value },
}

#[derive(Debug)]
pub struct NativeFunction {
    pub ident: &'static str,
    pub function: fn(&mut Vm) -> Value,
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        self.ident == other.ident && self.function == other.function
    }
}

#[derive(Debug, PartialEq)]
pub struct Closure {
    pub function: PrototypeRef,
    pub upvalues: Vec<UpvalueRef>,
    pub num_upvalues: usize,
}

impl Closure {
    pub fn new(function: PrototypeRef) -> Self {
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
    Module(ModuleRef),
}

impl Value {
    pub fn is_false(&self) -> bool {
        match self {
            Value::Unit | Value::Bool(false) | Value::Integer(0) => true,
            _ => false,
        }
    }

    pub fn as_string(self) -> Option<StringRef> {
        match self {
            Value::String(string) => Some(string),
            _ => None,
        }
    }

    pub fn as_array(self) -> Option<ArrayRef> {
        match self {
            Value::Array(array) => Some(array),
            _ => None,
        }
    }

    pub fn as_int(self) -> Option<i64> {
        match self {
            Value::Integer(integer) => Some(integer),
            _ => None,
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
            (Value::Integer(l), Value::Number(r)) => (*l as f64).partial_cmp(r),
            (Value::Number(l), Value::Integer(r)) => l.partial_cmp(&(*r as f64)),
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
            Value::Number(num) => num.to_bits().hash(state),
            Value::String(str) => str.hash(state),
            Value::Table(table) => Rc::as_ptr(table).hash(state),
            Value::Function(function) => Rc::as_ptr(function).hash(state),
            Value::Closure(closure) => Rc::as_ptr(closure).hash(state),
            Value::Array(array) => Rc::as_ptr(array).hash(state),
            Value::Module(module) => Rc::as_ptr(module).hash(state),
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
            Value::Function(function) => {
                write!(f, "fn {}: {:x?}", function.ident, function.function)
            }
            Value::Closure(closure) => write!(
                f,
                "fn {}: {:x?}",
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
            Value::Module(module) => {
                write!(f, "mod {}: {:x?}", module.ident, Rc::as_ptr(module))
            }
        }
    }
}
