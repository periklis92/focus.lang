use std::{collections::HashMap, ops::Deref};

use crate::gc::{GcObject, GcRef};

pub enum Value {
    Unit,
    Bool(bool),
    Char(char),
    Integer(i64),
    Number(f64),
    String(GcRef<String>),
    Array(GcRef<Array>),
    Table(GcRef<Table>),
}

pub struct String(std::string::String);

impl GcObject for String {
    fn mark(&self, gc: &mut crate::gc::Gc) {
        todo!()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Deref for String {
    type Target = std::string::String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Array(Vec<Value>);

impl GcObject for Array {
    fn mark(&self, gc: &mut crate::gc::Gc) {
        todo!()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Deref for Array {
    type Target = Vec<Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Table(HashMap<Value, Value>);

impl GcObject for Table {
    fn mark(&self, gc: &mut crate::gc::Gc) {
        todo!()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Deref for Table {
    type Target = HashMap<Value, Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
