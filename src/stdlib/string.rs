use std::rc::Rc;

use crate::{
    state::{Module, NativeModuleBuilder},
    value::Value,
    vm::{RuntimeError, Vm},
};

use super::fmt::format_to_string;

fn from_format(vm: &mut Vm) -> Result<Value, RuntimeError> {
    if vm.top() != 1 {
        panic!("Incorrect number of arguments");
    }
    let args = vm.pop().as_table().unwrap();
    let format = format_to_string(args);
    Ok(Value::String(Rc::new(format)))
}

pub fn module() -> Module {
    NativeModuleBuilder::new("String")
        .with_function("from_format", from_format)
        .build()
}
