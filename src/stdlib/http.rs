use std::{cell::RefCell, collections::HashMap, rc::Rc};

use reqwest::blocking::{get, Response};

use crate::{
    state::{Module, NativeModuleBuilder},
    value::Value,
    vm::{RuntimeError, Vm},
};

pub fn get_(vm: &mut Vm) -> Result<Value, RuntimeError> {
    if vm.top() != 2 {
        return Err(RuntimeError::IncorrectNumberOfArguments);
    }

    let value = vm.pop().as_string().ok_or(RuntimeError::UnexpectedType)?;
    let response = get(&*value).map_err(|e| RuntimeError::Custom(e.to_string()))?;

    let mut ret = HashMap::new();
    ret.insert(
        Value::String(Rc::new("is_ok".to_string())),
        Value::Bool(response.status().is_success()),
    );
    ret.insert(
        Value::String(Rc::new("_data".to_string())),
        Value::UserData(Box::new(Rc::new(RefCell::new(Some(response))))),
    );

    Ok(Value::Table(Rc::new(RefCell::new(ret))))
}

pub fn json(vm: &mut Vm) -> Result<Value, RuntimeError> {
    if vm.top() != 2 {
        return Err(RuntimeError::IncorrectNumberOfArguments);
    }

    let value = vm.pop().as_table().ok_or(RuntimeError::UnexpectedType)?;
    let data = value.borrow();
    let data = data
        .get(&Value::String(Rc::new("_data".to_string())))
        .ok_or(RuntimeError::Custom("Missing field _data".to_string()))?
        .clone();
    let data = data.as_user_data().ok_or(RuntimeError::UnexpectedType)?;
    let data = data
        .downcast::<RefCell<Option<Response>>>()
        .map_err(|_| RuntimeError::Custom("Unable to cast".to_string()))?;

    let data = data.replace(None);

    let json = data
        .ok_or(RuntimeError::Custom(
            "Response data have been consumed".to_string(),
        ))?
        .json::<Value>()
        .map_err(|e| RuntimeError::Custom(e.to_string()))?;

    Ok(json)
}

pub fn module() -> Module {
    NativeModuleBuilder::new("Http")
        .with_function("get", get_)
        .with_function("json", json)
        .build()
}
