use core::panic;
use std::{cell::RefCell, error::Error, fmt::Display, rc::Rc, usize};

use crate::{
    compiler::CompilerError,
    op::OpCode,
    state::{Module, ModuleLoader, ModuleValue},
    stdlib,
    value::{Closure, ClosureRef, Function, Table, Upvalue, UpvalueRef, Value},
};

const NUM_FRAMES: usize = 64;
const STACK_SIZE: usize = u8::MAX as usize;

struct CallFrame {
    closure: ClosureRef,
    ip: usize,
    slot_offset: usize,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Vm {
    frames: Vec<CallFrame>,
    stack: Vec<Value>,
    open_upvalues: Vec<UpvalueRef>,
    module_loader: ModuleLoader,
    #[cfg(target_arch = "wasm32")]
    event_emitter: web_sys::EventTarget,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
#[cfg(target_arch = "wasm32")]
impl Vm {
    pub fn add_event_listener(&self, type_: &str, function: &js_sys::Function) {
        self.event_emitter
            .add_event_listener_with_callback(type_, function)
            .unwrap();
    }
}

#[cfg(target_arch = "wasm32")]
impl Vm {
    pub fn event_target(&mut self) -> &mut web_sys::EventTarget {
        &mut self.event_emitter
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
impl Vm {
    pub fn new(module_loader: ModuleLoader) -> Self {
        Self {
            frames: Vec::with_capacity(NUM_FRAMES),
            stack: Vec::with_capacity(STACK_SIZE * NUM_FRAMES),
            open_upvalues: Vec::new(),
            module_loader,
            #[cfg(target_arch = "wasm32")]
            event_emitter: web_sys::EventTarget::new().unwrap(),
        }
    }

    pub fn new_with_std() -> Self {
        let mut module_loader = ModuleLoader::new("");
        module_loader.add_modules(stdlib::modules());
        Self {
            frames: Vec::with_capacity(NUM_FRAMES),
            stack: Vec::with_capacity(STACK_SIZE * NUM_FRAMES),
            open_upvalues: Vec::new(),
            module_loader,
            #[cfg(target_arch = "wasm32")]
            event_emitter: web_sys::EventTarget::new().unwrap(),
        }
    }

    pub fn load_from_source(&mut self, ident: &str, source: &str) -> Result<usize, CompilerError> {
        self.module_loader.load_module_from_source(ident, source)
    }

    pub fn execute_module(&mut self, index: usize, ident: &str) -> Result<(), RuntimeError> {
        let module = self.module_loader.module_at(index).unwrap();
        let index = module.local(ident).unwrap();
        self.load_module(module)?;
        let closure = self.stack[index].clone().as_closure().unwrap();
        self.push(Value::Closure(closure.clone()));
        self.push(Value::Unit);
        self.call(closure, 1)?;
        self.run()
    }
}

impl Vm {
    fn load_module(&mut self, module: Rc<Module>) -> Result<(), RuntimeError> {
        let module = Rc::new(module);
        match &module.value {
            ModuleValue::Native(_) => return Err(RuntimeError::CannotLoadNativeModuleAtRuntime),
            ModuleValue::Normal(prototype) => {
                let main = prototype.clone();
                let closure = Rc::new(Closure::from_prototype(main));
                self.push(Value::Closure(closure.clone()));
                self.call(closure, 0)?;
                self.run()?;
            }
        }

        Ok(())
    }

    pub fn stack(&self) -> &[Value] {
        &self.stack
    }

    fn frame(&mut self) -> &CallFrame {
        self.frames.last().unwrap()
    }

    fn frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().unwrap()
    }

    fn run(&mut self) -> Result<(), RuntimeError> {
        loop {
            self.frame_mut().ip += 1;
            if self.frame_mut().ip
                > self
                    .frame()
                    .closure
                    .function
                    .prototype()
                    .unwrap()
                    .code
                    .len()
            {
                break;
            }
            let ip = self.frame_mut().ip - 1;
            let code = self.frame_mut().closure.function.prototype().unwrap().code[ip];

            match code {
                OpCode::LoadConst(index) => {
                    let value = self
                        .frame_mut()
                        .closure
                        .function
                        .prototype()
                        .unwrap()
                        .constant(index as usize)
                        .clone();
                    self.push(value);
                }
                OpCode::LoadUnit => {
                    self.push(Value::Unit);
                }
                OpCode::LoadTrue => {
                    self.push(Value::Bool(true));
                }
                OpCode::LoadFalse => {
                    self.push(Value::Bool(false));
                }
                OpCode::LoadInt(integer) => {
                    self.push(Value::Integer(integer as i64));
                }
                OpCode::GetLocal(slot) => {
                    let offset = self.frames.last().unwrap().slot_offset;
                    let entry = self.stack[offset + slot as usize].clone();
                    self.push(entry);
                }
                OpCode::GetUpvalue(index) => {
                    let upvalue = self.frame().closure.upvalues[index as usize].clone();
                    match &*upvalue.borrow() {
                        Upvalue::Open { slot } => {
                            let value = self.stack[*slot].clone();
                            self.push(value);
                        }
                        Upvalue::Closed { value } => {
                            self.push(value.clone());
                        }
                    };
                }
                OpCode::GetModule(index) => {
                    let module = &self.module_loader.module_at(index as usize).unwrap();
                    let value = Rc::clone(module);
                    self.push(Value::Module(value));
                }
                OpCode::GetTable => {
                    let key = self.pop();
                    let table = self.pop();
                    match table {
                        Value::Table(table) => {
                            let table = RefCell::borrow_mut(table.as_ref());
                            let value = table.get(&key).cloned().unwrap_or(Value::Unit);
                            self.push(value);
                        }
                        Value::Array(array) => {
                            if let Value::Integer(index) = key {
                                let array = array.borrow();
                                if index as usize >= array.len() {
                                    panic!("Out of bounds");
                                }
                                self.push(array[index as usize].clone());
                            } else {
                                panic!("Non integer value cannot index array");
                            }
                        }
                        Value::Module(module) => {
                            if let Value::Integer(integer) = key {
                                let value = match &module.value {
                                    ModuleValue::Native(native) => native[integer as usize].clone(),
                                    ModuleValue::Normal(prototype) => {
                                        let closure =
                                            Rc::new(Closure::from_prototype(prototype.clone()));
                                        self.push(Value::Closure(closure.clone()));
                                        self.call(closure.clone(), 0)?;
                                        let slot_offset = self.frame().slot_offset;
                                        let value =
                                            self.stack[slot_offset + integer as usize].clone();
                                        self.close_upvalues(
                                            self.frames.last().unwrap().slot_offset,
                                        );
                                        let frame = self.frames.pop().unwrap();
                                        let frame_offset = frame.slot_offset;
                                        self.stack.truncate(frame_offset);
                                        value
                                    }
                                };
                                self.push(value);
                            } else {
                                unreachable!()
                            }
                        }
                        _ => panic!("Unable to index value {table:?}"),
                    }
                }
                OpCode::SetLocal(slot) => {
                    let offset = self.frames.last().unwrap().slot_offset;
                    let front = self.stack.last().unwrap().clone();
                    self.stack[offset + slot as usize] = front;
                }
                OpCode::SetUpvalue(index) => {
                    let value = self.pop();
                    let upvalue = &self.frames.last_mut().unwrap().closure.upvalues[index as usize];
                    match *RefCell::borrow_mut(upvalue) {
                        Upvalue::Open { slot } => {
                            self.stack[slot] = value;
                        }
                        Upvalue::Closed { value: ref mut val } => {
                            *val = value;
                        }
                    }
                }
                OpCode::SetTable => {
                    let value = self.pop();
                    let key = self.pop();
                    let table = self.pop();
                    match table {
                        Value::Table(table) => {
                            let mut table = RefCell::borrow_mut(table.as_ref());
                            table.insert(key, value);
                        }
                        Value::Array(array) => {
                            if let Value::Integer(index) = key {
                                let mut array = (*array).borrow_mut();
                                if index as usize >= array.len() {
                                    for _ in array.len()..=index as usize {
                                        array.push(Value::Unit);
                                    }
                                }
                                array[index as usize] = value;
                            } else {
                                panic!("Non integer value cannot index array");
                            }
                        }
                        _ => panic!("Unable to index value {table:?}"),
                    }
                }
                OpCode::CreateList(size) => {
                    let mut array = Vec::with_capacity(size as usize);
                    for _ in 0..size {
                        let value = self.pop();
                        array.insert(0, value);
                    }
                    self.push(Value::Array(Rc::new(RefCell::new(array))));
                }
                OpCode::CreateTable(size) => {
                    let mut table = Table::new();
                    for _ in 0..size {
                        let value = self.pop();
                        let key = self.pop();
                        table.insert(key, value);
                    }
                    self.push(Value::Table(Rc::new(RefCell::new(table))));
                }
                OpCode::Closure(index) => {
                    let prototype = self
                        .frame()
                        .closure
                        .function
                        .prototype()
                        .unwrap()
                        .prototypes[index as usize]
                        .clone();

                    let mut closure = Closure::from_prototype(prototype.clone());

                    for i in 0..closure.num_upvalues {
                        let is_local = prototype.upvalues[i].is_local;
                        let index = prototype.upvalues[i].index;
                        if is_local {
                            let slot_offset = self.frames.last().unwrap().slot_offset;
                            let upvalue = self.capture_upvalue(slot_offset + index as usize);
                            closure.upvalues.push(upvalue);
                        } else {
                            let upvalue = self.frames.last().unwrap().closure.upvalues
                                [index as usize]
                                .clone();
                            closure.upvalues.push(upvalue);
                        }
                    }
                    self.push(Value::Closure(Rc::new(closure)));
                }
                OpCode::Add => {
                    let rhs = self.pop();
                    let lhs = self.pop();
                    match (lhs, rhs) {
                        (Value::Number(l), Value::Number(r)) => {
                            self.push(Value::Number(l + r));
                        }
                        (Value::Integer(l), Value::Integer(r)) => {
                            self.push(Value::Integer(l + r));
                        }
                        (Value::Integer(l), Value::Number(r)) => {
                            self.push(Value::Number(l as f64 + r));
                        }
                        (Value::Number(l), Value::Integer(r)) => {
                            self.push(Value::Number(l + r as f64));
                        }
                        (lhs, rhs) => panic!("invalid values: {lhs:?}, {rhs:?}"),
                    }
                }
                OpCode::Subtract => {
                    let rhs = self.pop();
                    let lhs = self.pop();
                    match (lhs, rhs) {
                        (Value::Number(l), Value::Number(r)) => {
                            self.push(Value::Number(l - r));
                        }
                        (Value::Integer(l), Value::Integer(r)) => {
                            self.push(Value::Integer(l - r));
                        }
                        (Value::Integer(l), Value::Number(r)) => {
                            self.push(Value::Number(l as f64 - r));
                        }
                        (Value::Number(l), Value::Integer(r)) => {
                            self.push(Value::Number(l - r as f64));
                        }
                        _ => todo!(),
                    }
                }
                OpCode::Divide => {
                    let rhs = self.pop();
                    let lhs = self.pop();
                    match (lhs, rhs) {
                        (Value::Number(l), Value::Number(r)) => {
                            self.push(Value::Number(l / r));
                        }
                        (Value::Integer(l), Value::Integer(r)) => {
                            self.push(Value::Integer(l / r));
                        }
                        (Value::Integer(l), Value::Number(r)) => {
                            self.push(Value::Number(l as f64 / r));
                        }
                        (Value::Number(l), Value::Integer(r)) => {
                            self.push(Value::Number(l / r as f64));
                        }
                        _ => todo!(),
                    }
                }
                OpCode::IDivide => {
                    let rhs = self.pop();
                    let lhs = self.pop();
                    match (lhs, rhs) {
                        (Value::Number(l), Value::Number(r)) => {
                            self.push(Value::Integer(l as i64 / r as i64));
                        }
                        (Value::Integer(l), Value::Integer(r)) => {
                            self.push(Value::Integer(l / r));
                        }
                        (Value::Integer(l), Value::Number(r)) => {
                            self.push(Value::Integer(l / r as i64));
                        }
                        (Value::Number(l), Value::Integer(r)) => {
                            self.push(Value::Integer(l as i64 / r));
                        }
                        _ => todo!(),
                    }
                }
                OpCode::Multiply => {
                    let rhs = self.pop();
                    let lhs = self.pop();
                    match (lhs, rhs) {
                        (Value::Number(l), Value::Number(r)) => {
                            self.push(Value::Number(l * r));
                        }
                        (Value::Integer(l), Value::Integer(r)) => {
                            self.push(Value::Integer(l * r));
                        }
                        (Value::Integer(l), Value::Number(r)) => {
                            self.push(Value::Number(l as f64 * r));
                        }
                        (Value::Number(l), Value::Integer(r)) => {
                            self.push(Value::Number(l * r as f64));
                        }
                        _ => todo!(),
                    }
                }
                OpCode::Modulus => {
                    let rhs = self.pop();
                    let lhs = self.pop();
                    match (lhs, rhs) {
                        (Value::Number(l), Value::Number(r)) => {
                            self.push(Value::Number(l % r));
                        }
                        (Value::Integer(l), Value::Integer(r)) => {
                            self.push(Value::Integer(l % r));
                        }
                        (Value::Integer(l), Value::Number(r)) => {
                            self.push(Value::Number(l as f64 % r));
                        }
                        (Value::Number(l), Value::Integer(r)) => {
                            self.push(Value::Number(l % r as f64));
                        }
                        _ => todo!(),
                    }
                }
                OpCode::Negate => {
                    let value = self.pop();
                    let result = match value {
                        Value::Integer(i) => Value::Integer(-i),
                        Value::Number(n) => Value::Number(-n),
                        _ => return Err(RuntimeError::NegateOperatorOnNonNumericValue),
                    };
                    self.push(result);
                }
                OpCode::Not => {
                    let value = self.pop();
                    let result = if value.is_false() {
                        Value::Bool(true)
                    } else {
                        Value::Bool(false)
                    };
                    self.push(result);
                }
                OpCode::Call(num_args) => {
                    let value = self
                        .stack
                        .iter()
                        .nth_back(num_args as usize)
                        .unwrap()
                        .clone();
                    match value {
                        Value::Closure(closure) => match &closure.function {
                            Function::Prototype(_) => self.call(closure, num_args as usize)?,
                            Function::Native(_) => self.call_native(closure, num_args as usize)?,
                        },
                        _ => return Err(RuntimeError::CannotCallNonCallableValue),
                    }
                }
                OpCode::CmpEq => {
                    let rhs = self.pop();
                    let lhs = self.pop();
                    if lhs == rhs {
                        self.push(Value::Bool(true));
                    } else {
                        self.push(Value::Bool(false));
                    }
                }
                OpCode::CmpNEq => {
                    let rhs = self.pop();
                    let lhs = self.pop();
                    if lhs != rhs {
                        self.push(Value::Bool(true));
                    } else {
                        self.push(Value::Bool(false));
                    }
                }
                OpCode::CmpLEq => {
                    let rhs = self.pop();
                    let lhs = self.pop();
                    if lhs <= rhs {
                        self.push(Value::Bool(true));
                    } else {
                        self.push(Value::Bool(false));
                    }
                }
                OpCode::CmpGEq => {
                    let rhs = self.pop();
                    let lhs = self.pop();
                    if lhs >= rhs {
                        self.push(Value::Bool(true));
                    } else {
                        self.push(Value::Bool(false));
                    }
                }
                OpCode::CmpGreater => {
                    let rhs = self.pop();
                    let lhs = self.pop();
                    if lhs > rhs {
                        self.push(Value::Bool(true));
                    } else {
                        self.push(Value::Bool(false));
                    }
                }
                OpCode::CmpLess => {
                    let rhs = self.pop();
                    let lhs = self.pop();
                    if lhs < rhs {
                        self.push(Value::Bool(true));
                    } else {
                        self.push(Value::Bool(false));
                    }
                }
                OpCode::CmpAnd => {
                    let rhs = self.pop();
                    let lhs = self.pop();
                    if lhs.is_false() && rhs.is_false() {
                        self.push(Value::Bool(false));
                    } else {
                        self.push(Value::Bool(true));
                    }
                }
                OpCode::CmpOr => {
                    let rhs = self.pop();
                    let lhs = self.pop();
                    if !lhs.is_false() || !rhs.is_false() {
                        self.push(Value::Bool(true));
                    } else {
                        self.push(Value::Bool(false));
                    }
                }
                OpCode::JumpIfFalse(location) => {
                    let value = self.pop();
                    if value.is_false() {
                        self.frames.last_mut().unwrap().ip += location as usize;
                    }
                }
                OpCode::Jump(location) => {
                    self.frames.last_mut().unwrap().ip += location as usize;
                }
                OpCode::CloseUpvalue(index) => {
                    let offset = self.frame().slot_offset;
                    self.close_upvalues(offset + index as usize);
                }
                OpCode::Pop => {
                    self.pop();
                }
                OpCode::Return => {
                    if self.frames.len() == 1 {
                        return Ok(());
                    } else {
                        let result = self.pop();
                        self.close_upvalues(self.frames.last().unwrap().slot_offset);
                        let frame = self.frames.pop().unwrap();
                        if self.frames.is_empty() {
                            return Ok(());
                        }

                        let frame_offset = frame.slot_offset;
                        self.stack.truncate(frame_offset);
                        self.push(result);
                        return Ok(());
                    }
                }
            }
        }

        Ok(())
    }

    fn capture_upvalue(&mut self, index: usize) -> UpvalueRef {
        for open_upvalue in self.open_upvalues.iter().rev() {
            match *open_upvalue.borrow() {
                Upvalue::Open { slot } if slot <= index => {
                    if slot == index {
                        return Rc::clone(open_upvalue);
                    } else {
                        break;
                    }
                }
                _ => {}
            }
        }

        let new_upvalue = Rc::new(RefCell::new(Upvalue::Open { slot: index }));
        self.open_upvalues.push(Rc::clone(&new_upvalue));

        new_upvalue
    }

    fn close_upvalues(&mut self, last: usize) {
        let mut i = self.open_upvalues.len();
        loop {
            if i < 1 {
                break;
            }

            let upvalue = &self.open_upvalues[i - 1];
            let location = match *upvalue.borrow() {
                Upvalue::Open { slot } => slot,
                _ => unreachable!("Closed upvalue in open upvalue list."),
            };
            if location < last {
                break;
            }
            upvalue.replace(Upvalue::Closed {
                value: self.stack[location].clone(),
            });
            i -= 1;
        }

        self.open_upvalues.truncate(i);
    }

    pub fn call(&mut self, closure: ClosureRef, num_args: usize) -> Result<(), RuntimeError> {
        if closure.function.prototype().unwrap().num_args != num_args {
            return Err(RuntimeError::IncorrectNumberOfArguments);
        }

        if self.frames.len() == usize::MAX {
            return Err(RuntimeError::StackOverflow);
        }

        let frame = CallFrame {
            closure,
            ip: 0,
            slot_offset: (self.stack.len() - num_args - 1),
        };
        self.frames.push(frame);
        self.run()
    }

    fn call_native(&mut self, closure: ClosureRef, num_args: usize) -> Result<(), RuntimeError> {
        if self.frames.len() == usize::MAX {
            return Err(RuntimeError::StackOverflow);
        }

        let frame = CallFrame {
            closure: closure.clone(),
            ip: 0,
            slot_offset: (self.stack.len() - num_args - 1),
        };
        self.frames.push(frame);
        let result = (closure.function.native().unwrap().function)(self)?;
        let frame = self.frames.pop().unwrap();
        self.pop();
        if self.frames.is_empty() {
            return Ok(());
        }

        let frame_offset = frame.slot_offset;
        self.stack.truncate(frame_offset);
        self.push(result);

        Ok(())
    }

    pub fn top(&mut self) -> usize {
        self.stack.len() - self.frame().slot_offset
    }

    pub fn push(&mut self, value: Value) -> usize {
        let index = self.stack.len();
        self.stack.push(value);
        index
    }

    pub fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    StackOverflow,
    IncorrectNumberOfArguments,
    NegateOperatorOnNonNumericValue,
    CannotCallNonCallableValue,
    CannotLoadNativeModuleAtRuntime,
    UnexpectedType,
}

impl Error for RuntimeError {}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::StackOverflow => write!(f, "Stack Overflow"),
            RuntimeError::IncorrectNumberOfArguments => {
                write!(f, "Incorrect Number of Arguments")
            }
            RuntimeError::NegateOperatorOnNonNumericValue => {
                write!(f, "Negate operator on non numeric value")
            }
            RuntimeError::CannotCallNonCallableValue => write!(f, "Cannot call non callable value"),
            RuntimeError::CannotLoadNativeModuleAtRuntime => {
                write!(f, "Cannot load native module at runtime")
            }
            RuntimeError::UnexpectedType => {
                write!(f, "Unexpected type")
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl Into<wasm_bindgen::JsValue> for RuntimeError {
    fn into(self) -> wasm_bindgen::JsValue {
        wasm_bindgen::JsValue::from_str(&self.to_string())
    }
}
