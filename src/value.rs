use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

pub enum Value {
    Unit,
    Bool(bool),
    Integer(i64),
    Number(f64),
    String(Rc<String>),
    Table(Rc<RefCell<Table>>),
}

pub type Table = BTreeMap<String, Value>;
