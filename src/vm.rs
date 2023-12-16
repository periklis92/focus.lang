use core::panic;
use std::{borrow::BorrowMut, cell::RefCell, rc::Rc, usize};

use crate::{
    compiler::{Compiler, CompilerState},
    op::OpCode,
    state::Module,
    value::{Closure, ClosureRef, FunctionRef, Table, Upvalue, UpvalueRef, Value},
};

const NUM_FRAMES: usize = 64;
const STACK_SIZE: usize = u8::MAX as usize;

struct CallFrame {
    closure: ClosureRef,
    ip: usize,
    slot_offset: usize,
}

pub struct Vm {
    frames: Vec<CallFrame>,
    stack: Vec<Value>,
    states: Vec<CompilerState>,
    open_upvalues: Vec<UpvalueRef>,
    modules: Vec<Rc<Module>>,
}

impl Vm {
    pub fn new(states: Vec<CompilerState>) -> Self {
        Self {
            frames: Vec::with_capacity(NUM_FRAMES),
            stack: Vec::with_capacity(STACK_SIZE * NUM_FRAMES),
            states,
            open_upvalues: Vec::new(),
            modules: Vec::new(),
        }
    }

    pub fn with_modules(mut self, modules: Vec<Module>) -> Self {
        for module in modules {
            self.modules.push(Rc::new(module));
        }
        self
    }

    pub fn interpret(&mut self) {
        let main = Rc::new(self.states[0].prototype.clone());
        let closure = Rc::new(Closure::new(main));
        self.push(Value::Closure(Rc::clone(&closure)));
        self.call(closure, 0);
        self.run();
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

    fn run(&mut self) {
        loop {
            self.frame_mut().ip += 1;
            if self.frame_mut().ip > self.frame_mut().closure.function.code.len() {
                break;
            }
            let ip = self.frame_mut().ip - 1;
            let code = self.frame_mut().closure.function.code[ip];

            match code {
                OpCode::LoadConst(index) => {
                    let value = self
                        .frame_mut()
                        .closure
                        .function
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
                    let entry = self.stack.get(offset + slot as usize).unwrap().clone();
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
                    let module = &self.modules[index as usize];
                    let value = Rc::clone(module);
                    self.push(Value::Module(value));
                }
                OpCode::GetTable => {
                    let key = self.pop();
                    let table = self.pop();
                    match table {
                        Value::Table(table) => {
                            let table = RefCell::borrow_mut(table.as_ref());
                            let table = table.get(&key).cloned().unwrap_or(Value::Unit);
                            self.push(table);
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
                            if let Value::String(string) = key {
                                let local =
                                    module.locals.iter().find(|l| l.ident == *string).unwrap();
                                self.push(local.value.clone());
                            } else {
                                panic!("Non string key cannot index module");
                            }
                        }
                        _ => todo!(),
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
                        _ => todo!(),
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
                    let function = self.states[index as usize].prototype.clone();
                    let mut closure = Closure::new(Rc::new(function));

                    for i in 0..closure.num_upvalues {
                        let is_local = self.states[index as usize].upvalues[i].is_local;
                        let index = self.states[index as usize].upvalues[i].index;
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
                        _ => panic!(),
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
                        Value::Closure(closure) => {
                            self.call(closure, num_args as usize);
                        }
                        Value::Function(function) => {
                            self.call_native(function, num_args as usize);
                        }
                        _ => panic!("Cannot call non closure value ({value:?})"),
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
                OpCode::CloseUpvalue => {
                    let index = self.stack.len() - 1;
                    self.close_upvalues(index);
                    self.pop();
                }
                OpCode::Pop => {
                    self.pop();
                }
                OpCode::Return => {
                    if self.frames.len() == 1 {
                        return;
                    } else {
                        let result = self.pop();
                        self.close_upvalues(self.frames.last().unwrap().slot_offset);
                        let frame = self.frames.pop().unwrap();
                        if self.frames.is_empty() {
                            return;
                        }

                        let frame_offset = frame.slot_offset;
                        self.stack.truncate(frame_offset);
                        self.push(result);
                    }
                }
            }
        }
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
            upvalue.replace_with(|_| Upvalue::Closed {
                value: self.stack[location].clone(),
            });
            i -= 1;
        }

        self.open_upvalues.truncate(i);
    }

    fn call(&mut self, closure: ClosureRef, num_args: usize) {
        if closure.function.num_args != num_args {
            panic!("Incorrect number of arguments.");
        }

        if self.frames.len() == usize::MAX {
            panic!("Stack overflow: max number of frames.");
        }

        let frame = CallFrame {
            closure,
            ip: 0,
            slot_offset: (self.stack.len() - num_args - 1),
        };
        self.frames.push(frame);
    }

    fn call_native(&mut self, function: FunctionRef, num_args: usize) {
        if self.frames.len() == usize::MAX {
            panic!("Stack overflow: max number of frames.");
        }

        let value = (function.function)(self);
        self.stack.drain(self.stack.len() - num_args..);
        self.push(value);
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
