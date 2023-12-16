use std::{cell::RefCell, rc::Rc};

use crate::value::Value;

fn format_to_string(args: Rc<RefCell<Vec<Value>>>) -> String {
    let args = args.borrow();
    let mut format = args[0].clone().as_string().unwrap().as_ref().to_owned();
    let mut offset = 0;
    for i in 1..args.len() {
        let arg = args[i].clone().as_array().unwrap();
        let arg = arg.borrow();
        let value = arg[0].to_string();
        let position = arg[1].clone().as_int().unwrap() as usize;
        format.insert_str(offset + position, &value);
        offset += value.len() + position;
    }
    format
}

pub mod string {
    use std::rc::Rc;

    use crate::{
        state::{Module, ModuleLocal},
        value::{NativeFunction, Value},
        vm::Vm,
    };

    use super::format_to_string;

    fn from_format(vm: &mut Vm) -> Value {
        let args = vm.pop().as_array().unwrap();
        let format = format_to_string(args);
        Value::String(Rc::new(format))
    }

    pub fn module() -> Module {
        Module {
            ident: "String".to_string(),
            locals: vec![ModuleLocal {
                ident: "from_format".to_string(),
                value: Value::Function(Rc::new(NativeFunction {
                    ident: "from_format",
                    function: from_format,
                })),
            }],
        }
    }
}

pub mod io {
    use std::rc::Rc;

    use crate::{
        state::{Module, ModuleLocal},
        value::{NativeFunction, Value},
        vm::Vm,
    };

    use super::format_to_string;

    fn print(vm: &mut Vm) -> Value {
        let arg = vm.pop();
        match arg {
            Value::String(string) => println!("{}", *string),
            Value::Array(array) => println!("{}", format_to_string(array)),
            _ => panic!("Invalid argument"),
        }
        Value::Unit
    }

    pub fn module() -> Module {
        Module {
            ident: "IO".to_string(),
            locals: vec![ModuleLocal {
                ident: "print".to_string(),
                value: Value::Function(Rc::new(NativeFunction {
                    ident: "print",
                    function: print,
                })),
            }],
        }
    }
}
