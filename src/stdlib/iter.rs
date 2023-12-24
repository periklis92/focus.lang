use std::{cell::RefCell, rc::Rc};

use crate::{
    state::{Module, NativeModuleBuilder},
    value::Value,
    vm::{RuntimeError, Vm},
};

fn map(vm: &mut Vm) -> Result<Value, RuntimeError> {
    if vm.top() - 1 != 2 {
        panic!("Invalid number of arguments.");
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
        results.push(vm.pop());
    }
    Ok(Value::Array(Rc::new(RefCell::new(results))))
}

fn filter(vm: &mut Vm) -> Result<Value, RuntimeError> {
    if vm.top() - 1 != 2 {
        panic!("Invalid number of arguments.");
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

pub fn module() -> Module {
    NativeModuleBuilder::new("Iter")
        .with_function("map", map)
        .with_function("filter", filter)
        .build()
}
