use std::{
    cell::{RefCell, RefMut},
    error::Error,
    fmt::Display,
    io::{BufWriter, Write},
    rc::Rc,
};

use crate::{
    ast::{
        ArithmeticOperator, BooleanOperator, ComparisonOperator, Expression, Import, ImportSource,
        Literal, Operation, PathPart, Statement, UnaryOperation,
    },
    op::{FunctionIdx, InitLen, LocalIdx, OpCode},
    parser::{Parser, ParserError},
    state::{Local, Module, ModuleAlias, ModuleLoader, ModuleValue, Prototype, Upvalue},
    value::Value,
};

#[derive(Debug)]
pub struct ScopeResolver {
    locals: Vec<Local>,
    depth: usize,
    base_depth: usize,
}

impl ScopeResolver {
    pub fn new() -> Self {
        Self {
            locals: Vec::new(),
            depth: 0,
            base_depth: 0,
        }
    }

    pub fn with_depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self.base_depth = depth;
        self
    }

    pub fn begin_scope(&mut self) {
        self.depth += 1;
    }

    pub fn end_scope(&mut self) -> usize {
        let n = self.num_locals_defined_in_scope();
        if self.depth > self.base_depth {
            self.depth -= 1;
        }
        self.locals.truncate(self.locals.len() - n);
        n
    }

    pub fn num_locals_defined_in_scope(&self) -> usize {
        self.locals
            .iter()
            .rev()
            .take_while(|l| l.depth >= self.depth)
            .count()
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn num_locals(&self) -> usize {
        self.locals.len()
    }

    pub fn add_local(&mut self, ident: String) -> Result<usize, CompilerError> {
        if self.locals.len() > u8::MAX as usize {
            return Err(CompilerError::MaxNumberOfLocalsExceeded);
        }

        let local = Local {
            ident: ident.clone(),
            depth: self.depth,
            is_captured: false,
        };
        let index = self.locals.len();
        self.locals.push(local);
        Ok(index)
    }

    pub fn local(&self, index: usize) -> &Local {
        &self.locals[index]
    }

    pub fn local_mut(&mut self, index: usize) -> &mut Local {
        &mut self.locals[index]
    }

    pub fn resolve_local(&self, ident: &str) -> Option<usize> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.ident == ident {
                return Some(i);
            }
        }
        None
    }

    fn mark_captured(&mut self, index: usize) {
        self.locals[index].is_captured = true;
    }
}

#[derive(Debug)]
pub struct CompilerState {
    pub parent: Option<Rc<RefCell<CompilerState>>>,
    pub prototype: Prototype,
    pub resolver: ScopeResolver,
    pub defined_states: Vec<Rc<RefCell<CompilerState>>>,
}

impl CompilerState {
    pub fn new(ident: String, is_anonymous: bool) -> Self {
        Self {
            parent: None,
            resolver: ScopeResolver::new(),
            prototype: Prototype::new(ident, is_anonymous),
            defined_states: Vec::new(),
        }
    }

    pub fn with_depth(mut self, depth: usize) -> Self {
        self.resolver = self.resolver.with_depth(depth);
        self
    }

    pub fn with_parent(mut self, parent: Rc<RefCell<CompilerState>>) -> Self {
        self.parent = Some(parent);
        self
    }

    fn build_prototype(&self) -> Rc<Prototype> {
        let mut proto = self.prototype.clone();
        for state in &self.defined_states {
            let child_proto = state.borrow().build_prototype();
            proto.prototypes.push(child_proto);
        }
        Rc::new(proto)
    }

    fn dump(&self, w: &mut impl Write) {
        writeln!(w, "fn {}", self.prototype.ident()).unwrap();

        let mut last_line = 0;
        for (i, op) in self.prototype.op_codes().iter().enumerate() {
            let line = self.prototype.line(i);
            if last_line < line + 1 {
                last_line = line + 1;
                writeln!(w, "{last_line}:").unwrap();
            }
            writeln!(w, " {op}").unwrap();
        }

        writeln!(w, "\nLocals:").unwrap();
        for (i, l) in self.prototype.debug_info.locals.iter().enumerate() {
            writeln!(w, "{i}: ident: {}, depth: {}", l.ident, l.depth).unwrap();
        }

        writeln!(w, "\nUpvalues:").unwrap();
        for (i, u) in self.prototype.upvalues.iter().enumerate() {
            writeln!(w, "{i}: index: {} is_local: {}", u.index, u.is_local).unwrap();
        }

        writeln!(w, "\nConstants:").unwrap();
        for (i, c) in self.prototype.constants().iter().enumerate() {
            writeln!(w, "{i}: {c}").unwrap();
        }
        writeln!(w).unwrap();
    }
}

pub struct Compiler<'a> {
    pub state: Rc<RefCell<CompilerState>>,
    pub parser: Parser<'a>,
    pub module_locals: Vec<String>,
    pub module_provider: &'a mut ModuleLoader,
    pub module_aliases: Vec<ModuleAlias>,
    line_no: usize,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a str, module_provider: &'a mut ModuleLoader) -> Self {
        Self {
            state: Rc::new(RefCell::new(CompilerState::new("<main>".to_string(), true))),
            parser: Parser::new(source),
            module_locals: Vec::new(),
            module_provider,
            module_aliases: Vec::new(),
            line_no: 1,
        }
    }

    fn state(&self) -> std::cell::Ref<'_, CompilerState> {
        self.state.borrow()
    }

    fn state_mut(&self) -> RefMut<'_, CompilerState> {
        self.state.borrow_mut()
    }

    pub fn compile_module(mut self, ident: &str) -> Result<Module, CompilerError> {
        let mut statements = Vec::new();
        self.add_local("<main>".to_string())?;
        self.module_locals.push("<main>".to_string());
        loop {
            let statement = self.parser.parse();
            match statement {
                Ok(Statement::Import { .. }) => statements.push(statement.unwrap()),
                Ok(Statement::Let { ref ident, .. }) => {
                    self.add_local(ident.to_string())?;
                    statements.push(statement.unwrap());
                }
                Ok(Statement::Function { ref ident, .. }) => {
                    self.add_local(ident.to_string())?;
                    statements.push(statement.unwrap());
                }
                Err(ParserError::EndOfSource) => break,
                Err(e) => return Err(CompilerError::ParserError(e)),
                _ => unreachable!(),
            }
        }

        for statement in statements {
            self.module_statement(statement)?;
        }

        let prototype = self.state().build_prototype();
        Ok(Module::new(
            ident,
            ModuleValue::Normal(prototype),
            self.module_locals,
        ))
    }

    pub fn compile(&mut self) -> Result<(), CompilerError> {
        let mut expressions = Vec::new();
        loop {
            let expr = self.parser.parse();
            match expr {
                Ok(expression) => {
                    expressions.push(expression);
                }
                Err(ParserError::EndOfSource) => {
                    break;
                }
                Err(err) => {
                    return Err(CompilerError::ParserError(err));
                }
            }
        }
        self.add_local("<main>".to_string())?;
        for statement in expressions {
            self.module_statement(statement)?;
        }
        Ok(())
    }

    pub fn module_statement(&mut self, statement: Statement) -> Result<(), CompilerError> {
        match statement {
            Statement::Let {
                line_no,
                ident,
                value,
            } => {
                self.line_no = line_no;
                if let Some(expression) = value {
                    self.expression(expression)?;
                } else {
                    self.emit_code(OpCode::LoadUnit);
                }
                self.module_locals.push(ident);
                Ok(())
            }
            Statement::Function {
                line_no,
                ident,
                args,
                expr,
            } => {
                self.line_no = line_no;
                self.function(ident.clone(), args, expr, false)?;
                self.module_locals.push(ident);
                Ok(())
            }
            Statement::Import {
                line_no,
                source,
                imports,
            } => {
                self.line_no = line_no;
                let module_index = match source {
                    ImportSource::Module(_) => todo!(),
                    ImportSource::File(filename) => self.module_provider.load_module(filename),
                };

                let module = self.module_provider.module_at(module_index).unwrap();

                match imports.as_slice() {
                    [Import::All { alias: None }] => {
                        for (i, local) in module.locals.iter().enumerate() {
                            self.module_aliases.push(ModuleAlias {
                                ident: local.to_string(),
                                module_index,
                                local_index: i,
                            });
                        }
                    }
                    _ => {}
                }
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    pub fn statement(&mut self, statement: Statement) -> Result<(), CompilerError> {
        match statement {
            Statement::Let {
                line_no,
                ident,
                value,
            } => {
                self.line_no = line_no;
                if let Some(expression) = value {
                    self.expression(expression)?;
                } else {
                    self.emit_code(OpCode::LoadUnit);
                }
                self.add_local(ident)?;
                Ok(())
            }
            Statement::Function {
                line_no,
                ident,
                args,
                expr,
            } => {
                self.line_no = line_no;
                self.function(ident.clone(), args, expr, false)?;
                self.add_local(ident)?;
                Ok(())
            }
            Statement::Import {
                line_no,
                source,
                imports,
            } => {
                self.line_no = line_no;
                let module_index = match source {
                    ImportSource::Module(_) => todo!(),
                    ImportSource::File(filename) => self.module_provider.load_module(filename),
                };

                let module = self.module_provider.module_at(module_index).unwrap();

                match imports.as_slice() {
                    [Import::All { alias: None }] => {
                        for (i, local) in module.locals.iter().enumerate() {
                            self.module_aliases.push(ModuleAlias {
                                ident: local.to_string(),
                                module_index,
                                local_index: i,
                            });
                        }
                    }
                    _ => {}
                }
                Ok(())
            }
            Statement::Expression {
                expression,
                line_no,
            } => {
                self.line_no = line_no;
                self.expression(expression)?;
                Ok(())
            }
        }
    }

    fn expression(&mut self, expression: Expression) -> Result<(), CompilerError> {
        match expression {
            Expression::UnaryOperation { operand, operation } => {
                self.expression(*operand)?;
                match operation {
                    UnaryOperation::Not => self.emit_code(OpCode::Not),
                    UnaryOperation::Negate => self.emit_code(OpCode::Negate),
                }
                Ok(())
            }
            Expression::Operation {
                lhs,
                operation,
                rhs,
            } => match operation {
                Operation::Assignment => self.assignment(*lhs, *rhs),
                Operation::Arithmetic(operator) => {
                    self.expression(*lhs)?;
                    self.expression(*rhs)?;
                    self.compile_arithmetic_operator(operator);
                    Ok(())
                }
                Operation::Comparison(comparison) => {
                    self.expression(*lhs)?;
                    self.expression(*rhs)?;
                    match comparison {
                        ComparisonOperator::Less => self.emit_code(OpCode::CmpLess),
                        ComparisonOperator::LessEqual => self.emit_code(OpCode::CmpLEq),
                        ComparisonOperator::Equal => self.emit_code(OpCode::CmpEq),
                        ComparisonOperator::NotEqual => self.emit_code(OpCode::CmpNEq),
                        ComparisonOperator::GreaterEqual => self.emit_code(OpCode::CmpGEq),
                        ComparisonOperator::Greater => self.emit_code(OpCode::CmpGreater),
                    }
                    Ok(())
                }
                Operation::Boolean(boolean) => {
                    self.expression(*lhs)?;
                    self.expression(*rhs)?;
                    match boolean {
                        BooleanOperator::And => self.emit_code(OpCode::CmpAnd),
                        BooleanOperator::Or => self.emit_code(OpCode::CmpOr),
                    }
                    Ok(())
                }
                Operation::Concat => {
                    self.expression(*lhs)?;
                    self.expression(*rhs)?;
                    self.emit_code(OpCode::Concat);
                    Ok(())
                }
            },
            Expression::Array(array) => {
                let len = array.len();
                if len > InitLen::MAX as usize {
                    return Err(CompilerError::ListInitializerTooLong);
                }
                for expression in array {
                    self.expression(expression)?;
                }
                self.emit_code(OpCode::CreateList(len as InitLen));
                Ok(())
            }
            Expression::Table(table) => {
                let len = table.len();
                if len > InitLen::MAX as usize {
                    return Err(CompilerError::MapInitializerTooLong);
                }
                for entry in table {
                    self.expression(entry.key)?;
                    self.expression(entry.value)?;
                }
                self.emit_code(OpCode::CreateTable(len as InitLen));
                Ok(())
            }
            Expression::Literal(literal) => self.literal(literal),
            Expression::Block(block) => {
                self.begin_scope();
                let block_len = block.len() - 1;
                for (i, statement) in block.into_iter().enumerate() {
                    let is_expression = statement.is_expression();
                    let pop = !matches!(
                        statement,
                        Statement::Expression {
                            expression: Expression::Call { .. },
                            ..
                        }
                    );
                    let is_assignment = matches!(
                        statement,
                        Statement::Expression {
                            expression: Expression::Operation {
                                operation: Operation::Assignment,
                                ..
                            },
                            ..
                        }
                    );
                    self.statement(statement)?;
                    if i < block_len && is_expression && pop && !is_assignment {
                        self.emit_code(OpCode::Pop);
                    }
                    if i == block_len && is_assignment {
                        self.emit_code(OpCode::LoadUnit);
                    }
                }
                self.end_scope();
                Ok(())
            }
            Expression::Path { ident, parts } => {
                let mut getter = None;
                if let Some((g, _)) = self.resolve_name(&ident) {
                    self.emit_code(g);
                    getter = Some(g);
                } else if let Some(alias) = self.resolve_module_alias(&ident) {
                    self.emit_code(OpCode::GetModule(alias.module_index as u8));
                    self.constant(Value::Integer(alias.local_index as i64))?;
                    self.emit_code(OpCode::GetTable);
                } else if let Some(module) = self.resolve_module(&ident) {
                    let opcode = OpCode::GetModule(module as u8);
                    self.emit_code(opcode);
                    getter = Some(opcode);
                } else {
                    return Err(CompilerError::NameNotFound(ident));
                }
                for part in parts {
                    match part {
                        PathPart::Ident(ident) => match getter {
                            Some(OpCode::GetModule(i)) => self.constant(Value::Integer(
                                self.module_provider
                                    .module_at(i as usize)
                                    .unwrap()
                                    .local(&ident)
                                    .ok_or(CompilerError::NameNotFound(ident))?
                                    as i64,
                            ))?,
                            _ => self.constant(Value::String(Rc::new(ident)))?,
                        },
                        PathPart::Index(expression) => {
                            self.expression(expression)?;
                        }
                    }
                    self.emit_code(OpCode::GetTable);
                }
                Ok(())
            }
            Expression::Call { callee, args } => {
                self.expression(*callee)?;
                let num_args = args.len();
                if num_args > u8::MAX as usize {
                    return Err(CompilerError::MaxNumberOfArgsExceeded);
                }
                for arg in args {
                    self.expression(arg)?;
                }
                self.emit_code(OpCode::Call(num_args as u8));
                Ok(())
            }
            Expression::Function { args, expr } => {
                let func_name = format!("<anonymous>");
                self.function(func_name, args, *expr, true)?;
                Ok(())
            }
            Expression::If {
                condition,
                block,
                r#else,
            } => {
                self.expression(*condition)?;
                let then_location = self.emit_jump(OpCode::JumpIfFalse(0));
                self.expression(*block)?;
                let else_location = self.emit_jump(OpCode::Jump(0));
                self.patch_jump(then_location);
                if let Some(r#else) = r#else {
                    self.expression(*r#else)?;
                } else {
                    self.emit_code(OpCode::LoadUnit);
                }
                self.patch_jump(else_location);
                Ok(())
            }
            Expression::InterpolatedString { format, arguments } => {
                self.constant(Value::String(Rc::new("format".to_string())))?;
                self.constant(Value::String(Rc::new(format)))?;
                let instruction = OpCode::CreateList(arguments.len() as u8);
                self.constant(Value::String(Rc::new("args".to_string())))?;
                for arg in arguments {
                    self.constant(Value::String(Rc::new("arg".to_string())))?;
                    self.expression(arg.expression)?;
                    self.constant(Value::String(Rc::new("offset".to_string())))?;
                    self.constant(Value::Integer(arg.offset as i64))?;
                    self.emit_code(OpCode::CreateTable(2));
                }
                self.emit_code(instruction);
                self.emit_code(OpCode::CreateTable(2));
                Ok(())
            }
        }
    }

    fn literal(&mut self, literal: Literal) -> Result<(), CompilerError> {
        match literal {
            Literal::Unit => self.constant(Value::Unit),
            Literal::Bool(b) => self.constant(Value::Bool(b)),
            Literal::Char(c) => self.constant(Value::Char(c)),
            Literal::Integer(i) => self.constant(Value::Integer(i)),
            Literal::Number(n) => self.constant(Value::Number(n)),
            Literal::String(s) => self.constant(Value::String(Rc::new(s))),
        }
    }

    fn constant(&mut self, value: Value) -> Result<(), CompilerError> {
        let instruction = match value {
            Value::Unit => OpCode::LoadUnit,
            Value::Bool(b) => {
                if b {
                    OpCode::LoadTrue
                } else {
                    OpCode::LoadFalse
                }
            }
            Value::Integer(i) => {
                if i <= u8::MAX as i64 {
                    OpCode::LoadInt(i as u8)
                } else {
                    let index = self.add_constant(Value::Integer(i))?;
                    OpCode::LoadConst(index)
                }
            }
            Value::Number(n) => {
                let index = self.add_constant(Value::Number(n))?;
                OpCode::LoadConst(index)
            }
            Value::String(s) => {
                let index = self.add_constant(Value::String(s))?;
                OpCode::LoadConst(index)
            }
            _ => return Err(CompilerError::NotAValidConstant),
        };
        self.emit_code(instruction);
        Ok(())
    }

    fn add_constant(&mut self, value: Value) -> Result<u8, CompilerError> {
        let index = self
            .state_mut()
            .prototype
            .add_constant(value)
            .ok_or(CompilerError::MaxNumberOfConstsExceeded)?;
        Ok(index)
    }

    fn function(
        &mut self,
        ident: String,
        args: Vec<String>,
        expression: Expression,
        is_anonymous: bool,
    ) -> Result<(), CompilerError> {
        let index = self.state().defined_states.len();
        let new_state = Rc::new(RefCell::new(
            CompilerState::new(ident.clone(), is_anonymous)
                .with_depth(self.state().resolver.depth)
                .with_parent(self.state.clone()),
        ));
        self.state_mut().defined_states.push(new_state.clone());
        self.state = new_state;
        self.begin_scope();
        self.add_local(ident.clone())?;

        if args.is_empty() {
            self.add_local("".to_string())?;
            self.state_mut().prototype.num_args += 1;
        } else {
            for arg in args {
                self.add_local(arg)?;
                self.state_mut().prototype.num_args += 1;
                if self.state_mut().prototype.num_args > u8::MAX as usize {
                    return Err(CompilerError::MaxNumberOfArgsExceeded);
                }
            }
        }

        self.expression(expression)?;
        self.end_scope();
        self.emit_code(OpCode::Return);
        let old_state = self.state().parent.clone().unwrap();
        self.state = old_state;
        self.emit_code(OpCode::Closure(index as FunctionIdx));

        Ok(())
    }

    fn add_local(&mut self, ident: String) -> Result<usize, CompilerError> {
        let index = self.state_mut().resolver.add_local(ident)?;
        let local = self.state().resolver.local(index).clone();
        self.state_mut().prototype.add_local(local);
        Ok(index)
    }

    fn assignment(&mut self, lhs: Expression, rhs: Expression) -> Result<(), CompilerError> {
        match lhs {
            Expression::Path { ident, parts } => {
                let (getter, setter) = if let Some((g, s)) = self.resolve_name(&ident) {
                    (g, s)
                } else if let Some(module) = self.resolve_module(&ident) {
                    (OpCode::GetModule(module as u8), None)
                } else {
                    return Err(CompilerError::NameNotFound(ident.to_string()));
                };

                if parts.is_empty() {
                    self.expression(rhs)?;
                    if let Some(setter) = setter {
                        self.emit_code(setter);
                    } else {
                        return Err(CompilerError::CannotSetTheValueOfAModule);
                    }
                } else {
                    self.emit_code(getter);
                    let num_parts = parts.len();
                    for (i, part) in parts.into_iter().enumerate() {
                        match part {
                            PathPart::Ident(ident) => {
                                self.constant(Value::String(Rc::new(ident)))?;
                            }
                            PathPart::Index(expression) => {
                                self.expression(expression)?;
                            }
                        }
                        if i < num_parts - 1 {
                            self.emit_code(OpCode::GetTable);
                        }
                    }
                    self.expression(rhs)?;
                    self.emit_code(OpCode::SetTable);
                }
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn resolve_name(&mut self, ident: &str) -> Option<(OpCode, Option<OpCode>)> {
        let local = self.state().resolver.resolve_local(&ident);
        if let Some(local) = local {
            return Some((
                OpCode::GetLocal(local as LocalIdx),
                Some(OpCode::SetLocal(local as LocalIdx)),
            ));
        }

        let upvalue = self.resolve_upvalue(&ident, self.state.clone());
        if let Some(index) = upvalue {
            return Some((
                OpCode::GetUpvalue(index as LocalIdx),
                Some(OpCode::SetUpvalue(index as LocalIdx)),
            ));
        }

        return None;
    }

    fn resolve_module_alias(&self, ident: &str) -> Option<ModuleAlias> {
        self.module_aliases
            .iter()
            .find(|local| local.ident == ident)
            .cloned()
    }

    fn resolve_module(&self, ident: &str) -> Option<usize> {
        self.module_provider.module(ident)
    }

    fn resolve_upvalue(&self, ident: &str, state: Rc<RefCell<CompilerState>>) -> Option<usize> {
        let parent = state.borrow().parent.clone()?;
        let local = parent.borrow().resolver.resolve_local(ident);
        if let Some(local) = local {
            parent.borrow_mut().resolver.mark_captured(local);
            return self.add_upvalue(local, true);
        }

        if let Some(upvalue) = self.resolve_upvalue(ident, parent.borrow().parent.clone()?) {
            return self.add_upvalue(upvalue, false);
        }

        None
    }

    fn add_upvalue(&self, index: usize, is_local: bool) -> Option<usize> {
        let i = self
            .state()
            .prototype
            .upvalues
            .iter()
            .enumerate()
            .find(|(_, u)| u.index == index && u.is_local == is_local)
            .map(|(i, _)| i);
        if let Some(i) = i {
            Some(i)
        } else {
            let upvalue_index = self.state().prototype.upvalues.len();
            self.state_mut()
                .prototype
                .upvalues
                .push(Upvalue { index, is_local });
            Some(upvalue_index)
        }
    }

    fn compile_arithmetic_operator(&mut self, operator: ArithmeticOperator) {
        match operator {
            ArithmeticOperator::Add => self.emit_code(OpCode::Add),
            ArithmeticOperator::Subtract => self.emit_code(OpCode::Subtract),
            ArithmeticOperator::Divide => self.emit_code(OpCode::Divide),
            ArithmeticOperator::IDivide => self.emit_code(OpCode::IDivide),
            ArithmeticOperator::Multiply => self.emit_code(OpCode::Multiply),
            ArithmeticOperator::Modulus => self.emit_code(OpCode::Modulus),
        }
    }

    fn begin_scope(&mut self) {
        self.state_mut().resolver.begin_scope();
    }

    fn end_scope(&mut self) -> usize {
        let size = self.state().resolver.num_locals_defined_in_scope();
        let num_locals = self.state().resolver.num_locals();
        for i in 0..size {
            if self.state().resolver.locals[num_locals - 1 - i].is_captured {
                self.emit_code(OpCode::CloseUpvalue((num_locals - 1 - i) as u8));
            }
        }
        self.state_mut().resolver.end_scope();
        size
    }

    fn emit_jump(&mut self, op_code: OpCode) -> usize {
        let index = self.state().prototype.code.len();
        self.emit_code(op_code);
        self.emit_code(OpCode::ExtraArg(0));
        index
    }

    fn patch_jump(&mut self, index: usize) {
        let len = self.state().prototype.code.len() - 2 - index;
        {
            let code = &mut self.state_mut().prototype.code[index];
            match code {
                OpCode::Jump(ref mut index) => {
                    *index = len as u8;
                }
                OpCode::JumpIfFalse(ref mut index) => {
                    *index = len as u8;
                }
                _ => unreachable!(),
            }
        }
        let arg = &mut self.state_mut().prototype.code[index + 1];
        match arg {
            OpCode::ExtraArg(ref mut arg) => {
                *arg = (len >> 8) as u8;
            }
            _ => unreachable!(),
        }
    }

    fn emit_code(&mut self, op_code: OpCode) {
        let line = self.line_no;
        self.state_mut().prototype.push_op_code(op_code, line);
    }

    pub fn dump(&self) -> String {
        let mut buffer = BufWriter::new(Vec::new());
        for (i, state) in self.state().defined_states.iter().enumerate() {
            write!(buffer, "{i}: ").unwrap();
            state.borrow().dump(&mut buffer);
        }
        String::from_utf8(buffer.into_inner().unwrap()).unwrap()
    }
}

#[derive(Debug)]
pub enum CompilerError {
    ParserError(ParserError),
    MaxNumberOfConstsExceeded,
    NotImplemented,
    EndOfSource,
    UnexpectedLocalAssignment,
    UnexpectedExpression,
    ListInitializerTooLong,
    NameNotFound(String),
    MapInitializerTooLong,
    MaxNumberOfLocalsExceeded,
    MaxNumberOfArgsExceeded,
    NotAValidConstant,
    CannotSetTheValueOfAModule,
}

impl From<ParserError> for CompilerError {
    fn from(value: ParserError) -> Self {
        Self::ParserError(value)
    }
}

impl Error for CompilerError {}

impl Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilerError::ParserError(e) => write!(f, "Parser error: {e}"),
            CompilerError::MaxNumberOfConstsExceeded => {
                write!(f, "Max number of constants exceeded")
            }
            CompilerError::NotImplemented => write!(f, "Not implemented yet"),
            CompilerError::EndOfSource => write!(f, "End of source"),
            CompilerError::UnexpectedLocalAssignment => write!(f, "Unexpected local assignment"),
            CompilerError::UnexpectedExpression => write!(f, "Unexpected expression"),
            CompilerError::ListInitializerTooLong => write!(f, "List initializer too long"),
            CompilerError::NameNotFound(name) => write!(f, "Name `{name}` not found"),
            CompilerError::MapInitializerTooLong => write!(f, "Map initializer too long"),
            CompilerError::MaxNumberOfLocalsExceeded => write!(f, "Max number of locals exceeded"),
            CompilerError::MaxNumberOfArgsExceeded => write!(f, "Max number of args exceeded"),
            CompilerError::NotAValidConstant => write!(f, "Not a valid constant"),
            CompilerError::CannotSetTheValueOfAModule => {
                write!(f, "Cannot set the value in another module")
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl Into<wasm_bindgen::JsValue> for CompilerError {
    fn into(self) -> wasm_bindgen::JsValue {
        wasm_bindgen::JsValue::from_str(&self.to_string())
    }
}
