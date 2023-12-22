use std::rc::Rc;

use crate::value::{TableRef, Value};

pub fn format_to_string(args: TableRef) -> String {
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
