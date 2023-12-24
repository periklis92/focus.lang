use std::{io::Write, path::Path, rc::Rc};

use crate::{
    compiler::{Compiler, CompilerError},
    op::{ConstIdx, OpCode},
    stdlib,
    value::{Closure, NativeFunction, Value},
    vm::{RuntimeError, Vm},
};

#[derive(Debug, Clone)]
pub struct ModuleAlias {
    pub ident: String,
    pub module_index: usize,
    pub local_index: usize,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub struct ModuleLoader {
    modules: Vec<Rc<Module>>,
    #[cfg(not(target_arch = "wasm32"))]
    root: String,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
impl ModuleLoader {
    pub fn new(_root: &str) -> Self {
        Self {
            modules: vec![Rc::new(stdlib::io::module())],
            #[cfg(not(target_arch = "wasm32"))]
            root: _root.to_string(),
        }
    }

    pub fn load_module_from_source(
        &mut self,
        ident: &str,
        source: &str,
    ) -> Result<usize, CompilerError> {
        let compiler = Compiler::new(source, self);
        let module = Rc::new(compiler.compile_module(ident)?);
        let index = self.modules.len();
        self.modules.push(module.clone());
        Ok(index)
    }
}

impl ModuleLoader {
    pub fn add_module(&mut self, module: Module) {
        self.modules.push(Rc::new(module));
    }

    pub fn add_modules(&mut self, modules: Vec<Module>) {
        for module in modules {
            self.add_module(module);
        }
    }

    pub fn module(&self, ident: &str) -> Option<usize> {
        self.modules.iter().position(|m| m.ident == ident)
    }

    pub fn module_at(&self, index: usize) -> Option<Rc<Module>> {
        self.modules.get(index).cloned()
    }

    pub fn load_module(&mut self, path: impl AsRef<Path>) -> usize {
        let path = if path.as_ref().extension().is_none() {
            let mut buf = path.as_ref().to_path_buf();
            buf.set_extension("fl");
            buf.canonicalize().unwrap()
        } else {
            path.as_ref().canonicalize().unwrap()
        };
        let name = path.with_extension("");
        let name = name.file_name().unwrap().to_str().unwrap();
        let source = std::fs::read_to_string(path).unwrap();
        let compiler = Compiler::new(&source, self);
        let module = Rc::new(compiler.compile_module(name).unwrap());
        let index = self.modules.len();
        self.modules.push(module.clone());
        index
    }
}

#[derive(Debug, Clone)]
pub struct Local {
    pub ident: String,
    pub depth: usize,
    pub is_captured: bool,
}

#[derive(Debug, Clone)]
pub struct Upvalue {
    pub index: usize,
    pub is_local: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ModuleValue {
    Native(Vec<Value>),
    Normal(Rc<Prototype>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Module {
    pub ident: String,
    pub locals: Vec<String>,
    pub value: ModuleValue,
}

impl Module {
    pub fn new(name: &str, value: ModuleValue, locals: Vec<String>) -> Self {
        Self {
            ident: name.to_string(),
            locals,
            value,
        }
    }

    pub fn add_local(&mut self, ident: &str) {
        self.locals.push(ident.to_string());
    }

    pub fn local(&self, ident: &str) -> Option<usize> {
        self.locals.iter().position(|l| l == ident)
    }

    pub fn dump(&self, buf: &mut impl Write) -> Result<(), std::io::Error> {
        match &self.value {
            ModuleValue::Native(native) => {
                for (value, name) in native.iter().zip(&self.locals) {
                    writeln!(buf, "ident: {name} value: {value}")?;
                }
                Ok(())
            }
            ModuleValue::Normal(prototype) => prototype.dump(buf),
        }
    }
}

pub struct NativeModuleBuilder {
    pub ident: String,
    pub locals: Vec<String>,
    pub values: Vec<Value>,
}

impl NativeModuleBuilder {
    pub fn new(ident: &str) -> Self {
        Self {
            ident: ident.to_string(),
            locals: Vec::new(),
            values: Vec::new(),
        }
    }

    pub fn with_function<T: Fn(&mut Vm) -> Result<Value, RuntimeError> + 'static>(
        mut self,
        ident: &str,
        function: T,
    ) -> Self {
        self.locals.push(ident.to_string());
        self.values
            .push(Value::Closure(Rc::new(Closure::from_native(Rc::new(
                NativeFunction {
                    ident: ident.to_string(),
                    function: Rc::new(function),
                },
            )))));
        self
    }

    pub fn build(self) -> Module {
        Module {
            ident: self.ident,
            locals: self.locals,
            value: ModuleValue::Native(self.values),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Name {
    pub ident: String,
    pub start_pc: usize,
}

#[derive(Debug, Clone)]
pub struct DebugInfo {
    pub locals: Vec<Local>,
    pub lines: Vec<usize>,
}

impl DebugInfo {
    pub fn new() -> Self {
        Self {
            locals: Vec::new(),
            lines: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Prototype {
    pub code: Vec<OpCode>,
    pub constants: Vec<Value>,
    pub ident: String,
    pub num_args: usize,
    pub debug_info: DebugInfo,
    pub upvalues: Vec<Upvalue>,
    pub prototypes: Vec<Rc<Prototype>>,
    pub is_anonymous: bool,
}

impl Prototype {
    pub fn new(ident: String, is_anonymous: bool) -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            ident,
            num_args: 0,
            upvalues: Vec::new(),
            debug_info: DebugInfo::new(),
            prototypes: Vec::new(),
            is_anonymous,
        }
    }

    pub fn ident(&self) -> &str {
        &self.ident
    }
    pub fn line(&self, index: usize) -> usize {
        self.debug_info.lines[index]
    }

    pub fn push_op_code(&mut self, op_code: OpCode, line: usize) {
        self.code.push(op_code);
        self.debug_info.lines.push(line);
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

    pub fn add_local(&mut self, local: Local) {
        self.debug_info.locals.push(local);
    }

    pub fn constant(&self, index: usize) -> &Value {
        &self.constants[index]
    }

    pub fn constants(&self) -> &[Value] {
        &self.constants
    }

    pub fn dump(&self, buf: &mut impl Write) -> Result<(), std::io::Error> {
        writeln!(buf, "fn {}", self.ident())?;

        let mut last_line = 0;
        for (i, op) in self.op_codes().iter().enumerate() {
            let line = self.line(i);
            if last_line < line + 1 {
                last_line = line + 1;
                writeln!(buf, "{last_line}:")?;
            }
            writeln!(buf, " {op}")?;
        }

        writeln!(buf, "\nLocals:").unwrap();
        for (i, l) in self.debug_info.locals.iter().enumerate() {
            writeln!(buf, "{i}: ident: {}, depth: {}", l.ident, l.depth).unwrap();
        }

        writeln!(buf, "\nUpvalues:").unwrap();
        for (i, u) in self.upvalues.iter().enumerate() {
            writeln!(buf, "{i}: index: {} is_local: {}", u.index, u.is_local).unwrap();
        }

        writeln!(buf, "\nConstants:").unwrap();
        for (i, c) in self.constants().iter().enumerate() {
            writeln!(buf, "{i}: {c}").unwrap();
        }
        writeln!(buf)?;

        if !self.prototypes.is_empty() {
            writeln!(buf, "defined in: {}", self.ident)?;
            for proto in &self.prototypes {
                proto.dump(buf)?;
            }
        }

        Ok(())
    }
}

impl PartialEq for Prototype {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self as *const Prototype, other as *const Prototype)
    }
}
