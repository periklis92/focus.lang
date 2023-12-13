use std::{
    io::{BufWriter, Write},
    rc::Rc,
};

use crate::{
    ast::{
        ArithmeticOperator, BooleanOperator, ComparisonOperator, Expression, Literal, Operation,
        PathPart, Statement, UnaryOperation,
    },
    op::{FunctionIdx, InitLen, LocalIdx, OpCode},
    parser::{Parser, ParserError},
    state::Prototype,
    value::Value,
};

#[derive(Debug)]
pub struct Local {
    ident: String,
    depth: usize,
    is_captured: bool,
    is_initialized: bool,
}

#[derive(Debug)]
pub struct Upvalue {
    pub index: usize,
    pub is_local: bool,
}

#[derive(Debug)]
pub struct ScopeResolver {
    locals: Vec<Local>,
    depth: usize,
}

impl ScopeResolver {
    pub fn new() -> Self {
        Self {
            locals: Vec::new(),
            depth: 0,
        }
    }

    pub fn begin_scope(&mut self) {
        self.depth += 1;
    }

    pub fn end_scope(&mut self) -> usize {
        self.depth -= 1;

        let mut count = 0;
        while !self.locals.is_empty() && self.locals.last().unwrap().depth > self.depth {
            count += 1;
            self.locals.pop();
        }

        count
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
            ident,
            depth: self.depth,
            is_captured: false,
            is_initialized: false,
        };
        let index = self.locals.len();
        self.locals.push(local);
        Ok(index)
    }

    pub fn mark_initialized(&mut self, local_idx: usize) {
        self.locals[local_idx].is_initialized = true;
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
    pub prototype: Prototype,
    pub resolver: ScopeResolver,
    pub upvalues: Vec<Upvalue>,
}

impl CompilerState {
    pub fn new(ident: String) -> Self {
        Self {
            resolver: ScopeResolver::new(),
            prototype: Prototype::new(ident),
            upvalues: Vec::new(),
        }
    }

    fn dump(&self, w: &mut impl Write) {
        writeln!(w, "fn {}", self.prototype.ident()).unwrap();

        let mut last_line = 0;
        for (i, op) in self.prototype.op_codes().iter().enumerate() {
            /*let line = self.prototype.line(i);
            if last_line < line + 1 {
                last_line = line + 1;
                writeln!(w, "{last_line}:").unwrap();
            }*/
            writeln!(w, " {op}").unwrap();
        }

        /*writeln!(w, "\nLocals:").unwrap();
        for (i, l) in self.locals.iter().enumerate() {
            writeln!(w, "{i}: {}", l.ident).unwrap();
        }*/

        writeln!(w, "\nUpvalues:").unwrap();
        for (i, u) in self.upvalues.iter().enumerate() {
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
    pub states: Vec<CompilerState>,
    pub parser: Parser<'a>,
    pub current_state: usize,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            states: vec![CompilerState::new("<main>".to_string())],
            parser: Parser::new(source),
            current_state: 0,
        }
    }

    fn state(&self) -> &CompilerState {
        &self.states[self.current_state]
    }

    fn state_mut(&mut self) -> &mut CompilerState {
        &mut self.states[self.current_state]
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
        self.begin_scope();
        self.state_mut().resolver.add_local("<main>".to_string())?;
        for statement in expressions {
            self.statement(statement)?;
        }
        Ok(())
    }

    pub fn statement(&mut self, expression: Statement) -> Result<(), CompilerError> {
        match expression {
            Statement::Let { ident, value } => {
                let index = self.state_mut().resolver.add_local(ident)?;
                if let Some(expression) = value {
                    self.expression(expression)?;
                    self.state_mut().resolver.mark_initialized(index);
                }
                Ok(())
            }
            Statement::Function { ident, args, expr } => {
                self.function(ident.clone(), args, expr)?;
                self.state_mut().resolver.add_local(ident)?;
                Ok(())
            }
            Statement::Expression(expression) => self.expression(expression),
        }
    }

    fn expression(&mut self, expression: Expression) -> Result<(), CompilerError> {
        match expression {
            Expression::UnaryOperation { operand, operation } => {
                self.expression(*operand)?;
                match operation {
                    UnaryOperation::Not => self.add_instruction(OpCode::Not),
                    UnaryOperation::Negate => self.add_instruction(OpCode::Negate),
                }
                Ok(())
            }
            Expression::Operation {
                lhs,
                operation,
                rhs,
            } => match operation {
                Operation::Assignment => self.compile_assignment(*lhs, *rhs),
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
                        ComparisonOperator::Less => self.add_instruction(OpCode::CmpLess),
                        ComparisonOperator::LessEqual => self.add_instruction(OpCode::CmpLEq),
                        ComparisonOperator::Equal => self.add_instruction(OpCode::CmpEq),
                        ComparisonOperator::NotEqual => todo!(),
                        ComparisonOperator::GreaterEqual => self.add_instruction(OpCode::CmpGEq),
                        ComparisonOperator::Greater => self.add_instruction(OpCode::CmpGreater),
                    }
                    Ok(())
                }
                Operation::Boolean(boolean) => {
                    self.expression(*lhs)?;
                    self.expression(*rhs)?;
                    match boolean {
                        BooleanOperator::And => self.add_instruction(OpCode::CmpAnd),
                        BooleanOperator::Or => self.add_instruction(OpCode::CmpOr),
                    }
                    Ok(())
                }
                Operation::Binary(_) => todo!(),
            },
            Expression::Array(array) => {
                let len = array.len();
                if len > InitLen::MAX as usize {
                    return Err(CompilerError::ListInitializerTooLong);
                }
                for expression in array {
                    self.expression(expression)?;
                }
                self.add_instruction(OpCode::CreateList(len as InitLen));
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
                self.add_instruction(OpCode::CreateTable(len as InitLen));
                Ok(())
            }
            Expression::Literal(literal) => self.literal(literal),
            Expression::Block(block) => {
                let num_exprs = block.len();
                for (i, expression) in block.into_iter().enumerate() {
                    let should_pop = match &expression {
                        Statement::Let { .. } => false,
                        Statement::Expression(Expression::Operation { operation, .. })
                            if operation == &Operation::Assignment =>
                        {
                            false
                        }
                        Statement::Function { .. } => false,
                        _ => true,
                    };
                    self.statement(expression)?;
                    if i != num_exprs - 1 && should_pop {
                        self.add_instruction(OpCode::Pop);
                    } else if i == num_exprs - 1 {
                        let n = self.end_scope();
                        /*for _ in 0..n {
                            let resolver = &self.states.last().unwrap().resolver;
                            if resolver.locals.is_empty() {
                                break;
                            }
                            if resolver.locals[resolver.num_locals() - 1].is_captured {
                                self.add_instruction(OpCode::CloseUpvalue);
                            } else {
                                self.add_instruction(OpCode::Pop);
                            }
                        }*/
                        if !should_pop {
                            self.add_instruction(OpCode::LoadUnit);
                        }
                    }
                }
                Ok(())
            }
            Expression::Path { ident, parts } => {
                let (getter, _) = self.resolve_name(&ident)?;
                let num_parts = parts.len();
                if let OpCode::GetLocal(local) = getter {
                    if num_parts > 0
                        && !self
                            .state_mut()
                            .resolver
                            .local(local as usize)
                            .is_initialized
                    {
                        return Err(CompilerError::CannotUseUnitializedLocal);
                    }
                }
                self.add_instruction(getter);
                for (i, part) in parts.into_iter().enumerate() {
                    match part {
                        PathPart::Ident(ident) => {
                            let const_idx = self
                                .state_mut()
                                .prototype
                                .add_constant(Value::String(Rc::new(ident.clone())))
                                .ok_or(CompilerError::MaxNumberOfConstsExceeded)?;
                            self.add_instruction(OpCode::LoadConst(const_idx));
                        }
                        PathPart::Index(expression) => {
                            self.expression(expression)?;
                        }
                    }
                    self.add_instruction(OpCode::GetTable);
                }
                Ok(())
            }
            Expression::Call { callee: path, args } => {
                self.expression(*path)?;
                let num_args = args.len();
                if num_args > u8::MAX as usize {
                    return Err(CompilerError::MaxNumberOfArgsExceeded);
                }
                for arg in args {
                    self.expression(arg)?;
                }
                self.add_instruction(OpCode::Call(num_args as u8));
                Ok(())
            }
            Expression::Function { args, expr } => {
                let index = self.states.len();
                let func_name = format!("<anonymous`{index}>");
                self.function(func_name, args, *expr)?;
                Ok(())
            }
            Expression::If {
                condition,
                block,
                r#else,
            } => {
                self.expression(*condition)?;
                self.begin_scope();
                let then_location = self.state().prototype.code.len();
                self.add_instruction(OpCode::JumpIfFalse(0));
                self.expression(*block)?;
                let else_location = self.state().prototype.code.len();
                let diff = (else_location - then_location) as u8;
                match self.state_mut().prototype.code[then_location] {
                    OpCode::JumpIfFalse(ref mut l) => *l = diff,
                    _ => unreachable!(),
                }
                if let Some(r#else) = r#else {
                    self.begin_scope();
                    self.add_instruction(OpCode::Jump(0));
                    self.expression(*r#else)?;
                } else {
                    self.add_instruction(OpCode::LoadUnit);
                }
                let else_end = self.state().prototype.code.len();
                let diff = (else_end - else_location) as u8 - 1;
                match self.state_mut().prototype.code[else_location] {
                    OpCode::Jump(ref mut l) => *l = diff,
                    _ => unreachable!(),
                }
                Ok(())
            }
        }
    }

    fn literal(&mut self, literal: Literal) -> Result<(), CompilerError> {
        match literal {
            Literal::Unit => self.constant(Value::Unit),
            Literal::Bool(b) => self.constant(Value::Bool(b)),
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
        self.add_instruction(instruction);
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
    ) -> Result<(), CompilerError> {
        let index = self.states.len();
        self.states.push(CompilerState::new(ident.clone()));
        let previous_index = self.current_state;
        self.current_state = index;
        self.state_mut().resolver.depth = self.states[previous_index].resolver.depth;
        self.state_mut().resolver.begin_scope();
        self.state_mut().resolver.add_local(ident.clone())?;

        if args.is_empty() {
            self.state_mut().resolver.add_local("".to_string())?;
            self.state_mut().prototype.num_args += 1;
        } else {
            for arg in args {
                self.state_mut().resolver.add_local(arg)?;
                self.state_mut().prototype.num_args += 1;
                if self.state_mut().prototype.num_args > u8::MAX as usize {
                    return Err(CompilerError::MaxNumberOfArgsExceeded);
                }
            }
        }

        self.expression(expression)?;
        self.end_scope();
        self.add_instruction(OpCode::Return);
        self.current_state = previous_index;
        self.add_instruction(OpCode::Closure(index as FunctionIdx));

        Ok(())
    }

    fn resolve_name(&mut self, ident: &str) -> Result<(OpCode, OpCode), CompilerError> {
        if let Some(local) = self.state().resolver.resolve_local(&ident) {
            Ok((
                OpCode::GetLocal(local as LocalIdx),
                OpCode::SetLocal(local as LocalIdx),
            ))
        } else if let Some(upvalue) = self.resolve_upvalue(&ident, self.states.len() - 1) {
            Ok((
                OpCode::GetUpvalue(upvalue as LocalIdx),
                OpCode::SetUpvalue(upvalue as LocalIdx),
            ))
        } else {
            Err(CompilerError::LocalNotFound(ident.to_string()))
        }
    }

    fn compile_assignment(
        &mut self,
        lhs: Expression,
        rhs: Expression,
    ) -> Result<(), CompilerError> {
        match lhs {
            Expression::Path { ident, parts } => {
                let (getter, setter) = self.resolve_name(&ident)?;
                if parts.is_empty() {
                    self.expression(rhs)?;
                    if let OpCode::SetLocal(local) = setter {
                        self.state_mut().resolver.mark_initialized(local as usize);
                    }
                    self.add_instruction(setter);
                } else {
                    self.add_instruction(getter);
                    let num_parts = parts.len();
                    for (i, part) in parts.into_iter().enumerate() {
                        match part {
                            PathPart::Ident(ident) => {
                                let const_idx = self
                                    .state_mut()
                                    .prototype
                                    .add_constant(Value::String(Rc::new(ident.clone())))
                                    .ok_or(CompilerError::MaxNumberOfConstsExceeded)?;
                                self.add_instruction(OpCode::LoadConst(const_idx));
                            }
                            PathPart::Index(expression) => {
                                self.expression(expression)?;
                            }
                        }
                        if i < num_parts - 1 {
                            self.add_instruction(OpCode::GetTable);
                        }
                    }
                    self.expression(rhs)?;
                    self.add_instruction(OpCode::SetTable);
                }
            }
            _ => todo!(),
        }
        Ok(())
    }

    fn resolve_upvalue(&mut self, ident: &str, depth: usize) -> Option<usize> {
        if depth == 0 {
            return None;
        }

        let state = self
            .states
            .iter_mut()
            .rev()
            .find(|s| s.resolver.depth <= depth)?;

        if let Some(local) = state.resolver.resolve_local(ident) {
            state.resolver.mark_captured(local);
            return self.add_upvalue(depth, local, true);
        }

        if let Some(upvalue) = self.resolve_upvalue(ident, depth - 1) {
            return self.add_upvalue(depth, upvalue, false);
        }

        None
    }

    fn add_upvalue(&mut self, depth: usize, index: usize, is_local: bool) -> Option<usize> {
        let state = &mut self.states[depth];
        if let Some((i, _)) = state
            .upvalues
            .iter()
            .enumerate()
            .find(|(_, u)| u.index == index && u.is_local == is_local)
        {
            Some(i)
        } else {
            let upvalue_index = state.upvalues.len();
            state.upvalues.push(Upvalue { index, is_local });
            self.states[depth].prototype.upvalues += 1;
            Some(upvalue_index)
        }
    }

    fn compile_arithmetic_operator(&mut self, operator: ArithmeticOperator) {
        match operator {
            ArithmeticOperator::Add => self.add_instruction(OpCode::Add),
            ArithmeticOperator::Subtract => self.add_instruction(OpCode::Subtract),
            ArithmeticOperator::Divide => self.add_instruction(OpCode::Divide),
            ArithmeticOperator::IDivide => self.add_instruction(OpCode::IDivide),
            ArithmeticOperator::Multiply => self.add_instruction(OpCode::Multiply),
            ArithmeticOperator::Modulus => self.add_instruction(OpCode::Modulus),
        }
    }

    fn begin_scope(&mut self) {
        self.state_mut().resolver.begin_scope();
    }

    fn end_scope(&mut self) -> usize {
        self.state_mut().resolver.end_scope()
    }

    fn add_instruction(&mut self, op_code: OpCode) {
        let line = self.parser.lexer().line();
        self.state_mut().prototype.push_op_code(op_code, line);
    }

    pub fn dump(&self) -> String {
        let mut buffer = BufWriter::new(Vec::new());
        for (i, state) in self.states.iter().enumerate() {
            write!(buffer, "{i}: ").unwrap();
            state.dump(&mut buffer);
        }
        String::from_utf8(buffer.into_inner().unwrap()).unwrap()
    }
}

#[derive(Debug)]
pub enum CompilerError {
    ParserError(ParserError),
    MaxNumberOfRegistersExceeded,
    MaxNumberOfConstsExceeded,
    NotImplemented,
    UnknownToken,
    EndOfSource,
    UnexpectedLocalAssignment,
    UnexpectedExpression,
    ListInitializerTooLong,
    LocalNotFound(String),
    MapInitializerTooLong,
    CannotUseUnitializedLocal,
    MaxNumberOfLocalsExceeded,
    MaxNumberOfArgsExceeded,
    NotAValidConstant,
}

impl From<ParserError> for CompilerError {
    fn from(value: ParserError) -> Self {
        Self::ParserError(value)
    }
}
