use crate::{
    ast::{
        ArithmeticOperator, BinaryOperator, BooleanOperator, ComparisonOperator, Expression,
        Literal, Operation, PathPart, Statement, TableEntry, UnaryOperation,
    },
    lexer::Lexer,
    token::{Token, TokenType},
};

#[derive(Clone)]
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    last_expr_start_position: usize,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            lexer: Lexer::new(source),
            last_expr_start_position: 0,
        }
    }

    pub fn lexer(&self) -> &Lexer<'a> {
        &self.lexer
    }

    pub fn current_position(&self) -> usize {
        self.lexer.position()
    }

    pub fn current_position_in_line(&self) -> usize {
        self.current_position() - self.last_expr_start_position
    }

    pub fn get_last_expr_source(&self) -> &str {
        self.lexer
            .slice(self.last_expr_start_position..self.current_position())
    }

    pub fn get_last_expr_line(&self) -> &str {
        let source = self.lexer.source();
        let position = &source
            .chars()
            .skip(self.last_expr_start_position)
            .position(|c| c == '\n');
        if let Some(position) = position {
            &source[self.last_expr_start_position..self.last_expr_start_position + *position]
        } else {
            &source[self.last_expr_start_position..]
        }
    }

    fn expect(&mut self, token_type: TokenType) -> Result<Token, ParserError> {
        self.lexer
            .next_checked(token_type.clone())
            .ok_or(ParserError::UnexpectedToken(token_type, self.lexer.peek()))
    }

    fn expect_indented(&mut self, token_type: TokenType) -> Result<Token, ParserError> {
        self.lexer
            .next_checked_indented(token_type.clone())
            .ok_or(ParserError::UnexpectedToken(
                token_type,
                self.lexer.peek_indented().unwrap_or(TokenType::Unknown),
            ))
    }

    pub fn parse(&mut self) -> Result<Statement, ParserError> {
        let statement = self.statement()?;
        self.expect(TokenType::NewLine)?;
        Ok(statement)
    }

    fn statement(&mut self) -> Result<Statement, ParserError> {
        self.lexer.skip_new_lines();
        let token = self.lexer.peek();
        self.last_expr_start_position = self.lexer.position();
        match token {
            TokenType::Let => self.r#let(),
            TokenType::Function => self.function_statement(),
            TokenType::Eos => Err(ParserError::EndOfSource),
            TokenType::Unknown => Err(ParserError::UnknownToken),
            _ => Ok(Statement::Expression(self.expression()?)),
        }
    }

    fn expression(&mut self) -> Result<Expression, ParserError> {
        let mut lhs = self.primary()?;
        let mut previous_precedence = 0;
        loop {
            let operation = match self.operator() {
                Some(op) => op,
                None => return Ok(lhs),
            };
            let current_precedence = operation.precedence();
            if current_precedence < previous_precedence {
                return Ok(lhs);
            }

            self.lexer.next_indented();
            let indentation = self.lexer.indentation();
            if self.lexer.next_checked(TokenType::NewLine).is_some() {
                self.lexer.skip_new_lines();
                let next_indentation = self.lexer.peek_indentation();
                if next_indentation <= indentation {
                    return Err(ParserError::InvalidIndentation);
                }
            }

            let mut p = self.clone();
            p.primary()?;
            let next_operator = p.operator();

            let rhs = if next_operator.is_some_and(|op| op.precedence() > current_precedence) {
                previous_precedence += 1;
                self.expression()?
            } else {
                self.primary()?
            };

            lhs = Expression::Operation {
                lhs: lhs.into(),
                operation,
                rhs: rhs.into(),
            };
        }
    }

    fn primary(&mut self) -> Result<Expression, ParserError> {
        match self.lexer.peek() {
            TokenType::Unit => {
                self.lexer.next();
                Ok(Expression::Literal(Literal::Unit))
            }
            TokenType::Minus => {
                self.lexer.next();
                Ok(Expression::UnaryOperation {
                    operand: self.primary()?.into(),
                    operation: UnaryOperation::Negate,
                })
            }
            TokenType::Not => {
                self.lexer.next();
                Ok(Expression::UnaryOperation {
                    operand: self.primary()?.into(),
                    operation: UnaryOperation::Not,
                })
            }
            TokenType::Number => {
                let token = self.lexer.next();
                let mut num = self.lexer.slice(token.span).to_string();
                if num.contains('.') {
                    if num.ends_with('.') {
                        num.push('0');
                    }
                    num.parse::<f64>()
                        .map(|n| Expression::Literal(Literal::Number(n)))
                        .map_err(|e| ParserError::UnableToParseNumber(e))
                } else {
                    num.parse::<i64>()
                        .map(|n| Expression::Literal(Literal::Integer(n)))
                        .map_err(|e| ParserError::UnableToParseInt(e))
                }
            }
            TokenType::Function => self.function_expression(),
            TokenType::Ident => {
                if self.is_call()? {
                    self.call()
                } else {
                    self.path()
                }
            }
            TokenType::LParen => {
                let expr = if self.is_call()? {
                    self.lexer.next();
                    self.call()?
                } else {
                    self.lexer.next();
                    self.lexer.skip_new_lines();
                    self.expression()?
                };
                self.expect(TokenType::RParen)?;
                Ok(expr)
            }
            TokenType::LBracket => {
                self.lexer.next();
                self.lexer.skip_new_lines();
                let mut arr = Vec::new();
                while self.lexer.peek() != TokenType::RBracket
                    && self.lexer.peek() != TokenType::Eos
                {
                    let expr = self.primary()?;
                    arr.push(expr);
                    self.lexer.next_checked(TokenType::Comma);
                }
                self.expect(TokenType::RBracket)?;
                Ok(Expression::Array(arr))
            }
            TokenType::LCurly => self.table(),
            TokenType::DoubleQuote => Ok(Expression::Literal(Literal::String(self.string()?))),
            TokenType::True => {
                self.lexer.next();
                Ok(Expression::Literal(Literal::Bool(true)))
            }
            TokenType::False => {
                self.lexer.next();
                Ok(Expression::Literal(Literal::Bool(false)))
            }
            TokenType::If => {
                self.lexer.next();
                self.r#if()
            }
            _ => Err(ParserError::NotAPrimaryExpression),
        }
    }

    fn table(&mut self) -> Result<Expression, ParserError> {
        self.expect(TokenType::LCurly)?;
        let mut table = Vec::new();
        self.lexer.skip_new_lines();
        while self.lexer.peek() != TokenType::RCurly && self.lexer.peek() != TokenType::Eos {
            let key = match self.lexer.peek() {
                TokenType::LBracket => {
                    self.lexer.next();
                    let expr = self.expression()?;
                    self.expect(TokenType::RBracket)?;
                    expr
                }
                TokenType::Ident => {
                    let token = self.lexer.next();
                    Expression::Literal(Literal::String(self.lexer.slice(token.span).to_string()))
                }
                TokenType::DoubleQuote => Expression::Literal(Literal::String(self.string()?)),
                token => {
                    return Err(ParserError::UnexpectedTokenOneOf(
                        vec![TokenType::DoubleQuote, TokenType::Ident],
                        token,
                    ))
                }
            };
            self.expect(TokenType::Colon)?;
            self.lexer.skip_new_lines();
            let value = self.expression()?;
            table.push(TableEntry { key, value });
            self.lexer.skip_new_lines();
            if self.lexer.next_checked(TokenType::Comma).is_none() {
                break;
            }
        }
        self.expect(TokenType::RCurly)?;
        Ok(Expression::Table(table))
    }

    fn r#let(&mut self) -> Result<Statement, ParserError> {
        self.expect(TokenType::Let)?;
        let token = self.expect(TokenType::Ident)?;
        let ident = self.lexer.slice(token.span).to_string();
        let value = if self.lexer.next_checked(TokenType::Assign).is_none() {
            None
        } else {
            Some(self.expression()?)
        };
        Ok(Statement::Let { ident, value })
    }

    fn block(&mut self) -> Result<Expression, ParserError> {
        let statements = if self.lexer.peek() == TokenType::NewLine {
            let start_indentation = self.lexer.indentation();
            self.lexer.next(); // Skip new line

            self.block_with_indentation(start_indentation)?
        } else {
            vec![self.statement()?]
        };

        match statements.last() {
            Some(Statement::Let { .. } | Statement::Function { .. }) => {
                Err(ParserError::FoundStatementWhereExpressionWasExpected)
            }
            _ => Ok(Expression::Block(statements)),
        }
    }

    fn block_with_indentation(
        &mut self,
        indentation: usize,
    ) -> Result<Vec<Statement>, ParserError> {
        let mut block = Vec::new();
        while self.lexer.peek_indentation() > indentation {
            block.push(self.statement()?);
        }

        if block.is_empty() {
            Err(ParserError::ExpectedBlock)
        } else {
            Ok(block)
        }
    }

    fn is_call(&self) -> Result<bool, ParserError> {
        let mut cloned = self.clone();
        match cloned.lexer.peek() {
            TokenType::Ident => {
                cloned.path()?;
                Ok(cloned
                    .lexer
                    .next_indented()
                    .is_some_and(|t| t.token_type.is_primary() && t.token_type != TokenType::Minus))
            }
            TokenType::LParen => {
                cloned.lexer.next();
                cloned.expression()?;
                cloned.expect(TokenType::RParen)?;
                Ok(cloned
                    .lexer
                    .next_indented()
                    .is_some_and(|t| t.token_type.is_primary()))
            }
            _ => Ok(false),
        }
    }

    fn r#if(&mut self) -> Result<Expression, ParserError> {
        let if_indentation = self.lexer.indentation();
        let condition = self.expression()?.into();
        self.expect(TokenType::Then)?;
        let block = self.block()?.into();

        let r#else = if self
            .lexer
            .next_checked_continued(TokenType::Else, if_indentation)
            .is_some()
        {
            if self.lexer.next_checked(TokenType::If).is_some() {
                Some(self.r#if()?.into())
            } else {
                Some(self.block()?.into())
            }
        } else {
            None
        };

        Ok(Expression::If {
            condition,
            block,
            r#else,
        })
    }

    fn function_statement(&mut self) -> Result<Statement, ParserError> {
        self.expect(TokenType::Function)?;
        let token = self.expect(TokenType::Ident)?;
        let ident = self.lexer().slice(token.span).to_string();
        let args = self.function_args()?;
        self.expect(TokenType::ThinArrow)?;
        let expr = self.block()?.into();
        Ok(Statement::Function { ident, args, expr })
    }

    fn function_expression(&mut self) -> Result<Expression, ParserError> {
        self.expect(TokenType::Function)?;
        let args = self.function_args()?;
        self.expect(TokenType::ThinArrow)?;
        let expr = self.block()?.into();
        Ok(Expression::Function { args, expr })
    }

    fn function_args(&mut self) -> Result<Vec<String>, ParserError> {
        let mut args = Vec::new();
        while self
            .lexer
            .peek_indented()
            .is_some_and(|t| t != TokenType::ThinArrow)
        {
            let token = self.expect_indented(TokenType::Ident)?;
            let ident = self.lexer.slice(token.span).to_string();
            args.push(ident);
        }
        Ok(args)
    }

    fn callee(&mut self) -> Result<Expression, ParserError> {
        match self.lexer.peek() {
            TokenType::Ident => self.path(),
            TokenType::LParen => self.primary(),
            t => Err(ParserError::UnexpectedTokenOneOf(
                [TokenType::Ident, TokenType::LParen].to_vec(),
                t,
            )),
        }
    }

    fn call(&mut self) -> Result<Expression, ParserError> {
        let indentation = self.lexer.indentation();
        let mut call = self.call_simple()?;

        while self
            .lexer
            .next_checked_continued(TokenType::Pipe, indentation)
            .is_some()
        {
            match self.call_simple()? {
                Expression::Call { callee, mut args } => {
                    args.insert(0, call);
                    call = Expression::Call { callee, args }
                }
                _ => unreachable!(),
            }
        }

        Ok(call)
    }

    fn call_simple(&mut self) -> Result<Expression, ParserError> {
        let callee = self.callee()?.into();
        let mut args = Vec::new();

        while self.lexer.peek_indented().is_some_and(|t| t.is_primary()) {
            self.lexer.skip_new_lines();
            let arg = self.primary()?;
            args.push(arg);
        }

        Ok(Expression::Call { callee, args })
    }

    fn operator(&mut self) -> Option<Operation> {
        let token = self.lexer.peek_indented()?;
        match token {
            TokenType::Plus => Some(Operation::Arithmetic(ArithmeticOperator::Add)),
            TokenType::Minus => Some(Operation::Arithmetic(ArithmeticOperator::Subtract)),
            TokenType::Div => Some(Operation::Arithmetic(ArithmeticOperator::Divide)),
            TokenType::IDiv => Some(Operation::Arithmetic(ArithmeticOperator::IDivide)),
            TokenType::Mul => Some(Operation::Arithmetic(ArithmeticOperator::Multiply)),
            TokenType::Mod => Some(Operation::Arithmetic(ArithmeticOperator::Modulus)),
            TokenType::And => Some(Operation::Boolean(BooleanOperator::And)),
            TokenType::Or => Some(Operation::Boolean(BooleanOperator::Or)),
            TokenType::BinOr => Some(Operation::Binary(BinaryOperator::Or)),
            TokenType::BinAnd => Some(Operation::Binary(BinaryOperator::And)),
            TokenType::Lsh => Some(Operation::Binary(BinaryOperator::Lsh)),
            TokenType::Rsh => Some(Operation::Binary(BinaryOperator::Rsh)),
            TokenType::BinXor => Some(Operation::Binary(BinaryOperator::Xor)),
            TokenType::BinNot => Some(Operation::Binary(BinaryOperator::Not)),
            TokenType::Greater => Some(Operation::Comparison(ComparisonOperator::Greater)),
            TokenType::GreaterEqual => {
                Some(Operation::Comparison(ComparisonOperator::GreaterEqual))
            }
            TokenType::Equal => Some(Operation::Comparison(ComparisonOperator::Equal)),
            TokenType::NotEqual => Some(Operation::Comparison(ComparisonOperator::NotEqual)),
            TokenType::Less => Some(Operation::Comparison(ComparisonOperator::Less)),
            TokenType::LessEqual => Some(Operation::Comparison(ComparisonOperator::LessEqual)),
            TokenType::Assign => Some(Operation::Assignment),
            _ => None,
        }
    }

    fn string(&mut self) -> Result<String, ParserError> {
        self.expect(TokenType::DoubleQuote)?;
        let start_position = self.lexer.position();
        while self.lexer.peek() != TokenType::DoubleQuote && self.lexer.peek() != TokenType::Eos {
            if self.lexer.next_checked(TokenType::Eos).is_some() {
                return Err(ParserError::EarlyEos);
            }
            self.lexer.next();
        }
        self.expect(TokenType::DoubleQuote)?;
        let str = self.lexer.slice(start_position..self.lexer.position() - 1);
        Ok(str.to_string())
    }

    fn path(&mut self) -> Result<Expression, ParserError> {
        let token = self.expect(TokenType::Ident)?;
        let ident = self.lexer.slice(token.span).to_string();
        let mut path_parts = Vec::new();
        loop {
            match self.lexer.peek_empty() {
                TokenType::LBracket => {
                    self.lexer.next_empty();
                    let expr = self.primary()?;
                    self.expect(TokenType::RBracket)?;
                    path_parts.push(PathPart::Index(expr));
                }
                TokenType::Dot => {
                    self.lexer.next_empty();
                    if self.lexer.peek_empty() == TokenType::Empty {
                        return Err(ParserError::InvalidEmptySpace);
                    }
                    let token = self.expect(TokenType::Ident)?;
                    let ident = self.lexer.slice(token.span);
                    path_parts.push(PathPart::Ident(ident.to_string()));
                }
                _ => break,
            }
        }
        Ok(Expression::Path {
            ident,
            parts: path_parts,
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum ParserError {
    UnknownToken,
    EndOfSource,
    UnexpectedToken(TokenType, TokenType),
    UninitializedGlobal,
    ExpectedBlock,
    ReservedKeywordAsIdent,
    NotAPrimaryExpression,
    UnableToParseNumber(std::num::ParseFloatError),
    UnableToParseInt(std::num::ParseIntError),
    InvalidIndentation,
    UnexpectedTokenOneOf(Vec<TokenType>, TokenType),
    EarlyEos,
    InvalidEmptySpace,
    UnexpectedExpression(String),
    FoundStatementWhereExpressionWasExpected,
}

#[cfg(test)]
mod tests {
    use crate::ast::{
        ArithmeticOperator, Expression, Literal, Operation, PathPart, Statement, TableEntry,
    };

    use super::Parser;

    #[test]
    fn local_assignment() {
        let mut parser = Parser::new("let a");
        assert_eq!(
            parser.parse().expect("Unable to parse expression."),
            Statement::Let {
                ident: "a".to_string(),
                value: None
            }
        );
    }

    #[test]
    fn global_assignment() {
        let mut parser = Parser::new("a = 1 + 1");
        assert_eq!(
            parser.parse().expect("Unable to parse expression."),
            Statement::Expression(Expression::Operation {
                lhs: Expression::Path {
                    ident: "a".to_string(),
                    parts: vec![]
                }
                .into(),
                operation: Operation::Assignment,
                rhs: Expression::Operation {
                    lhs: Expression::Literal(Literal::Integer(1)).into(),
                    operation: Operation::Arithmetic(ArithmeticOperator::Add),
                    rhs: Expression::Literal(Literal::Integer(1)).into()
                }
                .into()
            })
        );
    }

    #[test]
    fn block() {
        let mut parser = Parser::new(
            r#"
            a = fn ->
                let a = 2
                3
            2
            "#,
        );
        assert_eq!(
            parser.parse().expect("Unable to parse expression."),
            Statement::Expression(Expression::Operation {
                lhs: Expression::Path {
                    ident: "a".to_string(),
                    parts: vec![]
                }
                .into(),
                operation: Operation::Assignment,
                rhs: Expression::Function {
                    args: vec![],
                    expr: Expression::Block(vec![
                        Statement::Let {
                            ident: "a".to_string(),
                            value: Some(Expression::Literal(Literal::Integer(2)).into())
                        },
                        Statement::Expression(Expression::Literal(Literal::Integer(3)))
                    ])
                    .into()
                }
                .into()
            })
        );
    }

    #[test]
    fn operation() {
        let mut parser = Parser::new("2 + 3 \n\t*\n\t\t4");
        assert_eq!(
            parser.parse().expect("Unable to parser operation."),
            Statement::Expression(Expression::Operation {
                lhs: Expression::Literal(Literal::Integer(2)).into(),
                operation: Operation::Arithmetic(ArithmeticOperator::Add),
                rhs: Expression::Operation {
                    lhs: Expression::Literal(Literal::Integer(3)).into(),
                    operation: Operation::Arithmetic(ArithmeticOperator::Multiply),
                    rhs: Expression::Literal(Literal::Integer(4)).into()
                }
                .into()
            })
        );
    }

    #[test]
    fn call() {
        let mut parser = Parser::new("some.function ()");
        assert_eq!(
            parser.parse().expect("Unable to parse expression."),
            Statement::Expression(Expression::Call {
                callee: Expression::Path {
                    ident: "some".to_string(),
                    parts: vec![PathPart::Ident("function".to_string())]
                }
                .into(),
                args: vec![Expression::Literal(Literal::Unit)]
            })
        )
    }

    #[test]
    fn table() {
        let mut parser = Parser::new("{hello: 1, test: call 2, \"with space\": 2.3, [1 + 1]: 2}");
        assert_eq!(
            parser.parse().expect("Unable to parse."),
            Statement::Expression(Expression::Table(vec![
                TableEntry {
                    key: Expression::Literal(Literal::String("hello".to_string())),
                    value: Expression::Literal(Literal::Integer(1))
                },
                TableEntry {
                    key: Expression::Literal(Literal::String("test".to_string())),
                    value: Expression::Call {
                        callee: Expression::Path {
                            ident: "call".to_string(),
                            parts: vec![]
                        }
                        .into(),
                        args: vec![Expression::Literal(Literal::Integer(2))]
                    }
                },
                TableEntry {
                    key: Expression::Literal(Literal::String("with space".to_string())),
                    value: Expression::Literal(Literal::Number(2.3))
                },
                TableEntry {
                    key: Expression::Operation {
                        lhs: Expression::Literal(Literal::Integer(1)).into(),
                        operation: Operation::Arithmetic(ArithmeticOperator::Add),
                        rhs: Expression::Literal(Literal::Integer(1)).into()
                    },
                    value: Expression::Literal(Literal::Integer(2))
                }
            ]))
        )
    }

    #[test]
    fn array() {
        let mut parser = Parser::new("[1, 2, 3, 4]");
        assert_eq!(
            parser.parse().expect("Unable to parse."),
            Statement::Expression(Expression::Array(vec![
                Expression::Literal(Literal::Integer(1)),
                Expression::Literal(Literal::Integer(2)),
                Expression::Literal(Literal::Integer(3)),
                Expression::Literal(Literal::Integer(4))
            ]))
        )
    }
}
