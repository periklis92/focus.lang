use std::fmt::Display;

pub type ConstIdx = u8;
pub type LocalIdx = u8;
pub type FunctionIdx = u8;
pub type InitLen = u8;

#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    LoadConst(ConstIdx),
    LoadUnit,
    LoadTrue,
    LoadFalse,
    LoadInt(u8),

    GetLocal(LocalIdx),
    GetUpvalue(LocalIdx),
    GetTable,

    SetLocal(LocalIdx),
    SetUpvalue(LocalIdx),
    SetTable,

    CreateList(InitLen),
    CreateTable(InitLen),

    Closure(FunctionIdx),

    Add,
    Subtract,
    Divide,
    IDivide,
    Multiply,
    Modulus,
    Negate,
    Not,

    CmpEq,
    CmpLess,
    CmpGreater,
    CmpLEq,
    CmpGEq,
    CmpAnd,
    CmpOr,

    JumpIfFalse(u8),
    Jump(u8),

    Call(u8),
    CloseUpvalue,
    Pop,
    Return,
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpCode::LoadConst(idx) => write!(f, "LoadConst {idx}"),
            OpCode::LoadUnit => write!(f, "LoadUnit"),
            OpCode::LoadTrue => write!(f, "LoadTrue"),
            OpCode::LoadFalse => write!(f, "LoadFalse"),
            OpCode::LoadInt(int) => write!(f, "LoadInt {int}"),
            OpCode::GetLocal(idx) => write!(f, "GetLocal {idx}"),
            OpCode::GetUpvalue(idx) => write!(f, "GetUpvalue {idx}"),
            OpCode::GetTable => write!(f, "GetTable"),
            OpCode::SetLocal(idx) => write!(f, "SetLocal {idx}"),
            OpCode::SetUpvalue(idx) => write!(f, "SetUpvalue {idx}"),
            OpCode::SetTable => write!(f, "SetTable"),
            OpCode::CreateList(len) => write!(f, "CreateList {len}"),
            OpCode::CreateTable(len) => write!(f, "CreateTable {len}"),
            OpCode::Closure(idx) => write!(f, "Closure {idx}"),
            OpCode::Add => write!(f, "Add"),
            OpCode::Subtract => write!(f, "Subtract"),
            OpCode::Divide => write!(f, "Divide"),
            OpCode::IDivide => write!(f, "IDivide"),
            OpCode::Multiply => write!(f, "Multiply"),
            OpCode::Modulus => write!(f, "Modulus"),
            OpCode::Negate => write!(f, "Negate"),
            OpCode::Not => write!(f, "Not"),
            OpCode::CmpEq => write!(f, "CmpEq"),
            OpCode::CmpLess => write!(f, "CmpLess"),
            OpCode::CmpGreater => write!(f, "CmpGreater"),
            OpCode::CmpLEq => write!(f, "CmpLEq"),
            OpCode::CmpGEq => write!(f, "CmpGEq"),
            OpCode::CmpAnd => write!(f, "CmpAnd"),
            OpCode::CmpOr => write!(f, "CmpOr"),
            OpCode::JumpIfFalse(location) => write!(f, "JumpIfFalse {location}"),
            OpCode::Jump(location) => write!(f, "Jump {location}"),
            OpCode::Call(args) => write!(f, "Call {args}"),
            OpCode::CloseUpvalue => write!(f, "CloseUpvalue"),
            OpCode::Pop => write!(f, "Pop"),
            OpCode::Return => write!(f, "Return"),
        }
    }
}
