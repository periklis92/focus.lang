use std::{cell::RefCell, io::Read, rc::Rc};

use crate::{
    state::{Module, NativeModuleBuilder},
    value::Value,
    vm::{RuntimeError, Vm},
};

use super::fmt::format_to_string;

fn print(vm: &mut Vm) -> Result<Value, RuntimeError> {
    let num_args = vm.top();
    let mut string = String::new();
    for _ in 1..num_args {
        let arg = vm.pop();
        string.insert_str(0, &arg.to_string());
    }
    #[cfg(target_arch = "wasm32")]
    {
        vm.event_target()
            .dispatch_event(
                &web_sys::CustomEvent::new_with_event_init_dict(
                    "log",
                    web_sys::CustomEventInit::new().detail(&string.clone().into()),
                )
                .unwrap(),
            )
            .unwrap();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("{}", string);
    }
    Ok(Value::Unit)
}

fn printf(vm: &mut Vm) -> Result<Value, RuntimeError> {
    if vm.top() - 1 != 1 {
        panic!("Invalid number of arguments");
    }
    let arg = vm.pop();
    match arg {
        Value::Table(table) => {
            let string = format_to_string(table);
            #[cfg(target_arch = "wasm32")]
            {
                vm.event_target()
                    .dispatch_event(
                        &web_sys::CustomEvent::new_with_event_init_dict(
                            "log",
                            web_sys::CustomEventInit::new().detail(&string.clone().into()),
                        )
                        .unwrap(),
                    )
                    .unwrap();
                web_sys::console::log_2(&"from rust: ".into(), &string.into());
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                println!("{}", string);
            }
        }
        _ => return Err(RuntimeError::UnexpectedType),
    }
    Ok(Value::Unit)
}

fn open_file(vm: &mut Vm) -> Result<Value, RuntimeError> {
    if vm.top() - 1 != 2 {
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

    Ok(Value::UserData(Box::new(Rc::new(RefCell::new(file)))))
}

fn read_file(vm: &mut Vm) -> Result<Value, RuntimeError> {
    if vm.top() - 1 != 1 {
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
    Ok(Value::String(Rc::new(buf)))
}

pub fn module() -> Module {
    NativeModuleBuilder::new("Io")
        .with_function("print", print)
        .with_function("printf", printf)
        .with_function("open_file", open_file)
        .with_function("read_file", read_file)
        .build()
}
