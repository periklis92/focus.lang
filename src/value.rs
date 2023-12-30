use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
    process::Termination,
    rc::Rc,
};

use crate::{
    state::{Module, Prototype},
    vm::{RuntimeError, Vm},
};

pub type Table = HashMap<Value, Value>;

pub type StringRef = Rc<String>;
pub type TableRef = Rc<RefCell<Table>>;
pub type PrototypeRef = Rc<Prototype>;
pub type NativeFunctionRef = Rc<NativeFunction>;
pub type UpvalueRef = Rc<RefCell<Upvalue>>;
pub type ClosureRef = Rc<Closure>;
pub type ArrayRef = Rc<RefCell<Vec<Value>>>;
pub type ModuleRef = Rc<Module>;
pub type UserDataRef = Box<Rc<dyn std::any::Any>>;

#[derive(Debug, PartialEq)]
pub enum Upvalue {
    Open { slot: usize },
    Closed { value: Value },
}

pub struct NativeFunction {
    pub ident: String,
    pub function: Rc<dyn Fn(&mut Vm) -> Result<Value, RuntimeError>>,
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        self.ident == other.ident && Rc::as_ptr(&self.function) == Rc::as_ptr(&other.function)
    }
}

impl Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeFunction")
            .field("ident", &self.ident)
            .field("function", &Rc::as_ptr(&self.function))
            .finish()
    }
}

#[derive(Debug, PartialEq)]
pub enum Function {
    Prototype(PrototypeRef),
    Native(NativeFunctionRef),
}

impl Function {
    pub fn ident(&self) -> &str {
        match self {
            Function::Prototype(prototype) => &prototype.ident,
            Function::Native(native) => &native.ident,
        }
    }

    pub fn prototype(&self) -> Option<PrototypeRef> {
        match self {
            Function::Prototype(prototype) => Some(prototype.clone()),
            _ => None,
        }
    }

    pub fn native(&self) -> Option<NativeFunctionRef> {
        match self {
            Function::Native(native) => Some(native.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Closure {
    pub function: Function,
    pub upvalues: Vec<UpvalueRef>,
    pub num_upvalues: usize,
}

impl Closure {
    pub fn from_prototype(function: PrototypeRef) -> Self {
        let num_upvalues = function.upvalues.len();
        Self {
            function: Function::Prototype(function),
            upvalues: Vec::with_capacity(num_upvalues),
            num_upvalues: num_upvalues,
        }
    }

    pub fn from_native(function: NativeFunctionRef) -> Self {
        Self {
            function: Function::Native(function),
            upvalues: Vec::new(),
            num_upvalues: 0,
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
    Closure(ClosureRef),
    Array(ArrayRef),
    Module(ModuleRef),
    UserData(UserDataRef),
}

impl Value {
    pub fn is_false(&self) -> bool {
        match self {
            Value::Unit | Value::Bool(false) | Value::Integer(0) => true,
            _ => false,
        }
    }

    pub fn as_user_data(self) -> Option<UserDataRef> {
        match self {
            Value::UserData(user_data) => Some(user_data),
            _ => None,
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

    pub fn as_table(self) -> Option<TableRef> {
        match self {
            Value::Table(table) => Some(table),
            _ => None,
        }
    }

    pub fn as_int(self) -> Option<i64> {
        match self {
            Value::Integer(integer) => Some(integer),
            _ => None,
        }
    }

    pub fn as_closure(self) -> Option<ClosureRef> {
        match self {
            Value::Closure(closure) => Some(closure),
            _ => None,
        }
    }

    pub fn type_name(&self) -> &str {
        match self {
            Value::Unit => "unit",
            Value::Bool(_) => "bool",
            Value::Integer(_) => "int",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Table(_) => "table",
            Value::Closure(_) => "function",
            Value::Array(_) => "array",
            Value::Module(_) => "module",
            Value::UserData(_) => "user_data",
        }
    }
}

impl Termination for Value {
    fn report(self) -> std::process::ExitCode {
        std::process::ExitCode::SUCCESS
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
            (Self::Closure(l0), Self::Closure(r0)) => l0 == r0,
            (Self::Array(l0), Self::Array(r0)) => l0 == r0,
            (Self::Module(l0), Self::Module(r0)) => l0 == r0,
            (Self::UserData(l0), Self::UserData(r0)) => Rc::as_ptr(l0) == Rc::as_ptr(r0),
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
            Value::Closure(closure) => Rc::as_ptr(closure).hash(state),
            Value::Array(array) => Rc::as_ptr(array).hash(state),
            Value::Module(module) => Rc::as_ptr(module).hash(state),
            Value::UserData(user_data) => Rc::as_ptr(user_data).hash(state),
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
            Value::UserData(user_data) => {
                write!(f, "user_data: {:x?}", Rc::as_ptr(user_data))
            }
        }
    }
}
