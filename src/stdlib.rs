use std::rc::Rc;

use crate::value::{TableRef, Value};

fn format_to_string(args: TableRef) -> String {
    let args = args.borrow();
    let mut format = args
        .get(&Value::String(Rc::new("format".to_string())))
        .unwrap()
        .clone()
        .as_string()
        .unwrap()
        .as_ref()
        .to_owned();
    let mut offset = 0;
    let args = args
        .get(&Value::String(Rc::new("args".to_string())))
        .unwrap()
        .clone()
        .as_array()
        .unwrap();
    let args = args.borrow();
    for i in 0..args.len() {
        let arg = args[i].clone().as_table().unwrap();
        let arg = arg.borrow();
        let value = arg
            .get(&Value::String(Rc::new("arg".to_string())))
            .unwrap()
            .to_string();
        let position = arg
            .get(&Value::String(Rc::new("offset".to_string())))
            .unwrap()
            .clone()
            .as_int()
            .unwrap() as usize;
        format.insert_str(offset + position, &value);
        offset += value.len() + position;
    }
    format
}

pub mod string {
    use std::rc::Rc;

    use crate::{
        state::{Module, NativeModuleBuilder},
        value::Value,
        vm::Vm,
    };

    use super::format_to_string;

    fn from_format(vm: &mut Vm) -> Value {
        if vm.top() != 1 {
            panic!("Incorrect number of arguments");
        }
        let args = vm.pop().as_table().unwrap();
        let format = format_to_string(args);
        Value::String(Rc::new(format))
    }

    pub fn module() -> Module {
        NativeModuleBuilder::new("String")
            .with_function("from_format", from_format)
            .build()
    }
}

pub mod io {
    use std::{cell::RefCell, io::Read, rc::Rc};

    use crate::{
        state::{Module, NativeModuleBuilder},
        value::Value,
        vm::Vm,
    };

    use super::format_to_string;

    fn print(vm: &mut Vm) -> Value {
        let num_args = vm.top();
        let mut string = String::new();
        for _ in 0..num_args {
            let arg = vm.pop();
            string.insert_str(0, &arg.to_string());
        }
        println!("{string}");
        Value::Unit
    }

    fn printf(vm: &mut Vm) -> Value {
        if vm.top() != 1 {
            panic!("Invalid number of arguments");
        }
        let arg = vm.pop();
        match arg {
            Value::Table(table) => println!("{}", format_to_string(table)),
            _ => panic!("Invalid type of value."),
        }
        Value::Unit
    }

    fn open_file(vm: &mut Vm) -> Value {
        if vm.top() != 2 {
            panic!("Invalid number of arguments");
        }

        let mode = vm.pop().as_string().unwrap();
        let path = vm.pop().as_string().unwrap();

        let append = mode.chars().any(|c| c == 'a');
        let create = mode.chars().any(|c| c == 'c');
        let truncate = mode.chars().any(|c| c == 't');
        let write = mode.chars().any(|c| c == 'w');
        let read = mode.chars().any(|c| c == 'r');

        let file = std::fs::File::options()
            .append(append)
            .create(create)
            .truncate(truncate)
            .write(write)
            .read(read)
            .open(&*path)
            .unwrap();

        return Value::UserData(Rc::new(RefCell::new(file)));
    }

    fn read_file(vm: &mut Vm) -> Value {
        if vm.top() != 1 {
            panic!("Invalid number of arguments");
        }

        let file = vm
            .pop()
            .as_user_data()
            .unwrap()
            .downcast::<RefCell<std::fs::File>>()
            .unwrap();

        let mut buf = String::new();
        file.borrow_mut().read_to_string(&mut buf).unwrap();
        Value::String(Rc::new(buf))
    }

    pub fn module() -> Module {
        NativeModuleBuilder::new("Io")
            .with_function("print", print)
            .with_function("printf", printf)
            .with_function("open_file", open_file)
            .with_function("read_file", read_file)
            .build()
    }
}

pub mod iter {
    use std::{cell::RefCell, rc::Rc};

    use crate::{
        state::{Module, NativeModuleBuilder},
        value::Value,
        vm::Vm,
    };

    fn map(vm: &mut Vm) -> Value {
        if vm.top() != 2 {
            panic!("Invalid number of arguments.");
        }
        let function = vm.pop().as_closure().unwrap();
        let value = &*vm.pop().as_array().unwrap();
        let value = value.borrow();
        vm.pop();
        let mut result = Vec::new();
        for v in value.iter() {
            vm.push(Value::Closure(function.clone()));
            vm.push(v.clone());
            vm.call(function.clone(), 1, true);
            result.push(vm.pop());
        }
        Value::Array(Rc::new(RefCell::new(result)))
    }

    pub fn module() -> Module {
        NativeModuleBuilder::new("Iter")
            .with_function("map", map)
            .build()
    }
}
