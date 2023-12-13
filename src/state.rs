use crate::{
    op::{ConstIdx, OpCode},
    value::Value,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Name {
    pub ident: String,
    pub start_pc: usize,
}

#[derive(Debug, Clone)]
pub struct Prototype {
    pub code: Vec<OpCode>,
    pub constants: Vec<Value>,
    pub ident: String,
    pub lines: Vec<usize>,
    pub num_args: usize,
    pub upvalues: usize,
}

impl Prototype {
    pub fn new(ident: String) -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            ident,
            lines: Vec::new(),
            num_args: 0,
            upvalues: 0,
        }
    }

    pub fn ident(&self) -> &str {
        &self.ident
    }
    pub fn line(&self, index: usize) -> usize {
        self.lines[index]
    }

    pub fn push_op_code(&mut self, op_code: OpCode, line: usize) {
        self.code.push(op_code);
        self.lines.push(line);
    }

    pub fn op_codes(&self) -> &[OpCode] {
        &self.code
    }

    pub fn add_constant(&mut self, value: Value) -> Option<ConstIdx> {
        if self.constants.len() > u8::MAX as usize {
            None
        } else if let Some(idx) = self.constants.iter().position(|v| v == &value) {
            Some(idx as ConstIdx)
        } else {
            let idx = self.constants.len();
            self.constants.push(value);
            Some(idx as ConstIdx)
        }
    }

    pub fn constant(&self, index: usize) -> &Value {
        &self.constants[index]
    }

    pub fn constants(&self) -> &[Value] {
        &self.constants
    }
}

impl PartialEq for Prototype {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self as *const Prototype, other as *const Prototype)
    }
}
