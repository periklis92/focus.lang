use std::rc::Rc;

use crate::{
    state::{Module, NativeModuleBuilder},
    value::Value,
    vm::{RuntimeError, Vm},
};

pub fn to_int(vm: &mut Vm) -> Result<Value, RuntimeError> {
    if vm.top() != 2 {
        return Err(RuntimeError::IncorrectNumberOfArguments);
    }

    let value = vm.pop();
    match value {
        Value::Bool(bool) => Ok(Value::Integer(bool as i64)),
        Value::Integer(int) => Ok(Value::Integer(int)),
        Value::Number(number) => Ok(Value::Integer(number as i64)),
        Value::String(str) => str
            .parse()
            .map(Value::Integer)
            .map_err(|_| RuntimeError::InvalidConversion),
        _ => Err(RuntimeError::UnexpectedType),
    }
}

pub fn to_string(vm: &mut Vm) -> Result<Value, RuntimeError> {
    if vm.top() != 2 {
        return Err(RuntimeError::IncorrectNumberOfArguments);
    }

    let value = vm.pop();
    Ok(Value::String(Rc::new(value.to_string())))
}

pub fn module() -> Module {
    NativeModuleBuilder::new("Parse")
        .with_function("to_int", to_int)
        .with_function("to_string", to_string)
        .build()
}
