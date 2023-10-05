use std::collections::BTreeMap;

use crate::{
    ast::{
        ArithmeticOperator, BinaryOperator, BooleanOperator, ComparisonOperator, Expression,
        Literal, Operation, PathPart, UnaryOperation,
    },
    lexer::Lexer,
    token::{Token, TokenType},
};

#[derive(Clone)]
pub struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            lexer: Lexer::new(source),
        }
    }

    fn expect(&mut self, token_type: TokenType) -> Result<Token, ParserError> {
        self.lexer
            .next_checked(token_type.clone())
            .ok_or(ParserError::UnexpectedToken(token_type, self.lexer.peek()))
    }

    pub fn parse(&mut self) -> Result<Expression, ParserError> {
        self.lexer.skip_new_lines();
        let token = self.lexer.peek();
        match token {
            TokenType::Eos => Err(ParserError::EndOfSource),
            TokenType::Unknown => Err(ParserError::UnknownToken),
            TokenType::Local => self.parse_assignment(),
            TokenType::Function => self.parse_function(),
            t if t.is_primary() => self.parse_operation(),
            t => Err(ParserError::UnexpectedTokenOneOf(
                vec![TokenType::Local, TokenType::Function],
                t,
            )),
        }
    }

    fn parse_table(&mut self) -> Result<Expression, ParserError> {
        self.expect(TokenType::LCurly)?;
        let mut map = BTreeMap::new();
        self.lexer.skip_new_lines();
        while self.lexer.peek() != TokenType::RBracket && self.lexer.peek() != TokenType::Eos {
            let key = self.parse_path_part()?;
            self.expect(TokenType::Colon)?;
            self.lexer.skip_new_lines();
            let value = self.parse()?;
            map.insert(key, value);
            if self.lexer.next_checked(TokenType::Comma).is_none() {
                self.lexer.skip_new_lines();
                self.expect(TokenType::RCurly)?;
                break;
            }
            self.lexer.skip_new_lines();
        }
        Ok(Expression::Table(map))
    }

    fn parse_assignment(&mut self) -> Result<Expression, ParserError> {
        self.expect(TokenType::Local)?;
        let ident_token = self.expect(TokenType::Ident)?;
        let ident = self.lexer.slice(ident_token.span).to_string();
        let value = if self.lexer.next_checked(TokenType::Assign).is_none() {
            None
        } else {
            Some(self.parse_block_or_expr()?.into())
        };
        Ok(Expression::Local { ident, value })
    }

    fn parse_block_or_expr(&mut self) -> Result<Expression, ParserError> {
        let expr = if self.lexer.peek() == TokenType::NewLine {
            let start_indentation = self.lexer.indentation();
            self.lexer.next(); // Skip new line
            self.parse_block(start_indentation)?
        } else {
            self.parse()?
        };

        match expr {
            Expression::Block(mut v) if v.len() == 1 => Ok(v.swap_remove(0)),
            _ => Ok(expr),
        }
    }

    fn parse_block(&mut self, indentation: usize) -> Result<Expression, ParserError> {
        let mut block = Vec::new();
        while self.lexer.peek_indentation() > indentation {
            block.push(self.parse()?);
        }

        if block.is_empty() {
            Err(ParserError::ExpectedBlock)
        } else {
            Ok(Expression::Block(block))
        }
    }

    fn parse_primary(&mut self) -> Result<Expression, ParserError> {
        match self.lexer.peek() {
            TokenType::Unit => {
                self.lexer.next();
                Ok(Expression::Literal(Literal::Unit))
            }
            TokenType::Minus => {
                self.lexer.next();
                Ok(Expression::UnaryOperation {
                    operand: self.parse_primary()?.into(),
                    operation: UnaryOperation::Negate,
                })
            }
            TokenType::Not => {
                self.lexer.next();
                Ok(Expression::UnaryOperation {
                    operand: self.parse_primary()?.into(),
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
            TokenType::Ident => {
                let token = self.lexer.next();
                let ident = self.lexer.slice(token.span);
                let path = self.parse_path(Expression::Ident(ident.to_string()))?;
                if self
                    .lexer
                    .peek_indented()
                    .is_some_and(|t| t.is_primary() && t != TokenType::Minus)
                {
                    self.parse_call(path)
                } else {
                    Ok(path)
                }
            }
            TokenType::Function => {
                self.lexer.next();
                self.parse_function_with_ident(None)
            }
            TokenType::LBracket => {
                self.lexer.next();
                self.lexer.skip_new_lines();
                let mut arr = Vec::new();
                while self.lexer.peek() != TokenType::RBracket
                    && self.lexer.peek() != TokenType::Eos
                {
                    let expr = self.parse_primary()?;
                    arr.push(expr);
                    self.lexer.next_checked(TokenType::Comma);
                }
                self.expect(TokenType::RBracket)?;
                Ok(Expression::Array(arr))
            }
            TokenType::LCurly => self.parse_table(),
            TokenType::DoubleQuote => {
                Ok(Expression::Literal(Literal::String(self.parse_string()?)))
            }
            _ => Err(ParserError::NotAPrimaryExpression),
        }
    }

    fn parse_integer(&mut self) -> Result<i64, ParserError> {
        let token = self.expect(TokenType::Number)?;
        let mut num = self.lexer.slice(token.span).to_string();
        num.parse::<i64>()
            .map_err(|e| ParserError::UnableToParseInt(e))
    }

    fn parse_float(&mut self) -> Result<f64, ParserError> {
        let token = self.expect(TokenType::Number)?;
        let mut num = self.lexer.slice(token.span).to_string();
        self.expect(TokenType::Dot)?;
        num.push('.');
        if self.lexer.peek_empty() == TokenType::Number {
            let token = self.lexer.next_empty();
            let n = self.lexer.slice(token.span);
            num.push_str(n);
        } else {
            num.push('0');
        }
        num.parse::<f64>()
            .map_err(|e| ParserError::UnableToParseNumber(e))
    }

    fn parse_function(&mut self) -> Result<Expression, ParserError> {
        self.expect(TokenType::Function)?;
        let token = self.expect(TokenType::Ident)?;
        let ident = self.lexer.slice(token.span).to_string();
        self.parse_function_with_ident(Some(ident))
    }

    fn parse_function_with_ident(
        &mut self,
        ident: Option<String>,
    ) -> Result<Expression, ParserError> {
        let mut args = Vec::new();
        while self
            .lexer
            .peek_indented()
            .is_some_and(|t| t != TokenType::ThinArrow)
        {
            let token = self.expect(TokenType::Ident)?;
            let ident = self.lexer.slice(token.span).to_string();
            args.push(ident);
        }
        self.lexer.next(); // Consume thin arrow
        let expr = self.parse_block_or_expr()?.into();
        Ok(Expression::Function { ident, args, expr })
    }

    fn parse_call(&mut self, path: Expression) -> Result<Expression, ParserError> {
        let mut args = Vec::new();

        while self.lexer.peek_indented().is_some_and(|t| t.is_primary()) {
            self.lexer.skip_new_lines();
            let arg = self.parse_primary()?;
            args.push(arg);
        }

        Ok(Expression::Call {
            path: path.into(),
            args,
        })
    }

    fn parse_operation(&mut self) -> Result<Expression, ParserError> {
        let mut lhs = self.parse_primary()?;
        let mut previous_precedence = 0;
        loop {
            let operation = match self.parse_operator() {
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
            p.parse_primary()?;
            let next_operator = p.parse_operator();

            let rhs = if next_operator.is_some_and(|op| op.precedence() > current_precedence) {
                previous_precedence += 1;
                self.parse_operation()?
            } else {
                self.parse_primary()?
            };

            lhs = Expression::Operation {
                lhs: lhs.into(),
                operation,
                rhs: rhs.into(),
            };
        }
    }

    fn parse_operator(&mut self) -> Option<Operation> {
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

    fn parse_string(&mut self) -> Result<String, ParserError> {
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

    fn parse_path(&mut self, expr: Expression) -> Result<Expression, ParserError> {
        let mut path_parts = Vec::new();
        loop {
            match self.lexer.peek_empty() {
                TokenType::LBracket => {
                    self.lexer.next_empty();
                    if self.lexer.peek_empty() == TokenType::Empty {
                        return Err(ParserError::InvalidEmptySpace);
                    }
                    path_parts.push(self.parse_path_part()?);
                    self.expect(TokenType::RBracket)?;
                }
                TokenType::Dot => {
                    let nxt = self.lexer.next_empty();
                    if self.lexer.peek_empty() == TokenType::Empty {
                        return Err(ParserError::InvalidEmptySpace);
                    }
                    path_parts.push(self.parse_path_part()?)
                }
                _ => break,
            }
        }
        Ok(Expression::Path {
            expr: expr.into(),
            parts: path_parts,
        })
    }

    fn parse_path_part(&mut self) -> Result<PathPart, ParserError> {
        match self.lexer.peek() {
            TokenType::DoubleQuote => Ok(PathPart::String(self.parse_string()?)),
            TokenType::Ident => {
                let token = self.lexer.next();
                let ident = self.lexer.slice(token.span).to_string();
                Ok(PathPart::Ident(ident))
            }
            TokenType::Number => Ok(PathPart::Number(self.parse_integer()?)),
            _ => Err(ParserError::UnexpectedTokenOneOf(
                vec![TokenType::LBracket, TokenType::Ident, TokenType::Number],
                self.lexer.next().token_type,
            )),
        }
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
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::ast::{ArithmeticOperator, Expression, Literal, Operation, PathPart};

    use super::Parser;

    #[test]
    fn local_assignment() {
        let mut parser = Parser::new("local a");
        assert_eq!(
            parser.parse().expect("Unable to parse expression."),
            Expression::Local {
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
            Expression::Operation {
                lhs: Expression::Path {
                    expr: Expression::Ident("a".to_string()).into(),
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
            }
        );
    }

    #[test]
    fn block() {
        let mut parser = Parser::new(
            r#"
            a = fn ->
                local a = 2
                3
            2
            "#,
        );
        assert_eq!(
            parser.parse().expect("Unable to parse expression."),
            Expression::Operation {
                lhs: Expression::Path {
                    expr: Expression::Ident("a".to_string()).into(),
                    parts: vec![]
                }
                .into(),
                operation: Operation::Assignment,
                rhs: Expression::Function {
                    ident: None,
                    args: vec![],
                    expr: Expression::Block(vec![
                        Expression::Local {
                            ident: "a".to_string(),
                            value: Some(Expression::Literal(Literal::Integer(2)).into())
                        },
                        Expression::Literal(Literal::Integer(3))
                    ])
                    .into()
                }
                .into()
            }
        );
    }

    #[test]
    fn operation() {
        let mut parser = Parser::new("2 + 3 \n\t*\n\t\t4");
        assert_eq!(
            parser.parse().expect("Unable to parser operation."),
            Expression::Operation {
                lhs: Expression::Literal(Literal::Integer(2)).into(),
                operation: Operation::Arithmetic(ArithmeticOperator::Add),
                rhs: Expression::Operation {
                    lhs: Expression::Literal(Literal::Integer(3)).into(),
                    operation: Operation::Arithmetic(ArithmeticOperator::Multiply),
                    rhs: Expression::Literal(Literal::Integer(4)).into()
                }
                .into()
            }
        );
    }

    #[test]
    fn call() {
        let mut parser = Parser::new("some.function ()");
        assert_eq!(
            parser.parse().expect("Unable to parse expression."),
            Expression::Call {
                path: Expression::Path {
                    expr: Expression::Ident("some".to_string()).into(),
                    parts: vec![PathPart::Ident("function".to_string())]
                }
                .into(),
                args: vec![Expression::Literal(Literal::Unit)]
            }
        )
    }

    #[test]
    fn table() {
        let mut parser =
            Parser::new("{hello: 1, test: call 2, \"with space\": 2.3, 1: \"number\"}");
        assert_eq!(
            parser.parse().expect("Unable to parse."),
            Expression::Table(BTreeMap::from([
                (
                    PathPart::Ident("hello".to_string()),
                    Expression::Literal(Literal::Integer(1))
                ),
                (
                    PathPart::Ident("test".to_string()),
                    Expression::Call {
                        path: Expression::Path {
                            expr: Expression::Ident("call".into()).into(),
                            parts: vec![]
                        }
                        .into(),
                        args: vec![Expression::Literal(Literal::Integer(2))]
                    }
                ),
                (
                    PathPart::String("with space".to_string()),
                    Expression::Literal(Literal::Number(2.3))
                ),
                (
                    PathPart::Number(1),
                    Expression::Literal(Literal::String("number".to_string()))
                )
            ]))
        )
    }

    #[test]
    fn array() {
        let mut parser = Parser::new("[1, 2, 3, 4]");
        assert_eq!(
            parser.parse().expect("Unable to parse."),
            Expression::Array(vec![
                Expression::Literal(Literal::Integer(1)),
                Expression::Literal(Literal::Integer(2)),
                Expression::Literal(Literal::Integer(3)),
                Expression::Literal(Literal::Integer(4))
            ])
        )
    }
}
