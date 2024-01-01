use std::{cell::RefCell, rc::Rc};

use crate::{
    state::{Module, NativeModuleBuilder},
    value::{Closure, NativeFunction, Value},
    vm::{RuntimeError, Vm},
};

fn iter_from_fn<T: FnMut(&mut Vm) -> Result<Value, RuntimeError> + 'static>(fun: T) -> Value {
    Value::Iterator(Rc::new(Closure::from_native(Rc::new(NativeFunction {
        ident: "_iter".to_string(),
        function: Rc::new(RefCell::new(fun)),
    }))))
}

fn new(vm: &mut Vm) -> Result<Value, RuntimeError> {
    if vm.top() != 2 {
        return Err(RuntimeError::IncorrectNumberOfArguments);
    }

    let value = vm.pop();

    let result = match value {
        Value::Unit => todo!(),
        Value::Bool(_) => todo!(),
        Value::Integer(_) => todo!(),
        Value::Number(_) => todo!(),
        Value::String(str) => {
            let mut i = 0;
            iter_from_fn(move |_vm| {
                if i < str.len() {
                    let result = str.chars().nth(i).unwrap();
                    i += 1;
                    Ok(Value::Char(result))
                } else {
                    Ok(Value::Unit)
                }
            })
        }
        Value::Table(_) => todo!(),
        Value::Closure(closure) => Value::Iterator(closure),
        Value::Array(array) => {
            let mut i = 0;
            iter_from_fn(move |_vm| {
                let array = array.borrow();
                if i < array.len() {
                    let result = array[i].clone();
                    i += 1;
                    Ok(result)
                } else {
                    Ok(Value::Unit)
                }
            })
        }
        Value::Module(_) => todo!(),
        Value::UserData(_) => todo!(),
        Value::Char(_) => todo!(),
        Value::Iterator(iterator) => Value::Iterator(iterator),
    };

    Ok(result)
}

fn map(vm: &mut Vm) -> Result<Value, RuntimeError> {
    if vm.top() - 1 != 2 {
        return Err(RuntimeError::IncorrectNumberOfArguments);
    }
    let function = vm.pop().as_closure().unwrap();
    let value = vm.pop();
    vm.pop();
    let mut results = Vec::new();
    match value {
        Value::Array(array) => {
            let value = array.borrow();
            for v in value.iter() {
                vm.push(Value::Closure(function.clone()));
                vm.push(v.clone());
                vm.call(function.clone(), 1)?;
                results.push(vm.pop());
            }
            return Ok(Value::Array(Rc::new(RefCell::new(results))));
        }
        Value::Iterator(iterator) => loop {
            loop {
                vm.push(Value::Closure(iterator.clone()));
                vm.push(Value::Unit);
                vm.call(iterator.clone(), 1)?;
                let value = vm.pop();
                match value {
                    Value::Unit => break,
                    value => {
                        vm.push(Value::Closure(function.clone()));
                        vm.push(value.clone());
                        vm.call(function.clone(), 1)?;
                        results.push(vm.pop());
                    }
                }
            }
            return Ok(Value::Array(Rc::new(RefCell::new(results))));
        },
        _ => return Err(RuntimeError::UnexpectedType),
    }
}

fn filter(vm: &mut Vm) -> Result<Value, RuntimeError> {
    if vm.top() - 1 != 2 {
        return Err(RuntimeError::IncorrectNumberOfArguments);
    }
    let function = vm.pop().as_closure().unwrap();
    let value = &*vm.pop().as_array().unwrap();
    let value = value.borrow();
    vm.pop();
    let mut results = Vec::new();
    for v in value.iter() {
        vm.push(Value::Closure(function.clone()));
        vm.push(v.clone());
        vm.call(function.clone(), 1)?;
        let result = vm.pop();
        if result != Value::Unit {
            results.push(result);
        }
    }
    Ok(Value::Array(Rc::new(RefCell::new(results))))
}

fn for_each(vm: &mut Vm) -> Result<Value, RuntimeError> {
    if vm.top() - 1 != 2 {
        return Err(RuntimeError::IncorrectNumberOfArguments);
    }
    let function = vm.pop().as_closure().unwrap();
    let value = vm.pop();
    vm.pop();
    match value {
        Value::Array(array) => {
            let value = array.borrow();
            for v in value.iter() {
                vm.push(Value::Closure(function.clone()));
                vm.push(v.clone());
                vm.call(function.clone(), 1)?;
                vm.pop();
            }
            return Ok(Value::Unit);
        }
        Value::Iterator(iterator) => loop {
            loop {
                vm.push(Value::Closure(iterator.clone()));
                vm.push(Value::Unit);
                vm.call(iterator.clone(), 1)?;
                let value = vm.pop();
                match value {
                    Value::Unit => break,
                    value => {
                        vm.push(Value::Closure(function.clone()));
                        vm.push(value.clone());
                        vm.call(function.clone(), 1)?;
                        vm.pop();
                    }
                }
            }
            return Ok(Value::Unit);
        },
        _ => return Err(RuntimeError::UnexpectedType),
    }
}

pub fn module() -> Module {
    NativeModuleBuilder::new("Iter")
        .with_function("new", new)
        .with_function("map", map)
        .with_function("filter", filter)
        .with_function("for_each", for_each)
        .build()
}
