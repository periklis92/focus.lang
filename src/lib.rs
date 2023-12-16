pub mod ast;
pub mod compiler;
mod lexer;
mod op;
pub mod parser;
mod state;
pub mod stdlib;
mod token;
mod value;
pub mod vm;

#[cfg(target_arch = "wasm32")]
const ASD: i32 = 2;
