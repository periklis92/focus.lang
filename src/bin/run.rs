use std::{fs::File, io::Write, path::Path};

use focus_third::{
    compiler::{Compiler, CompilerError},
    parser::ParserError,
    vm::Vm,
};

#[derive(Debug)]
enum RunCliError {
    MissingInput,
    ReadWriteError(std::io::Error),
    FileError(std::io::Error),
    CompilerError(CompilerError),
}

impl From<CompilerError> for RunCliError {
    fn from(value: CompilerError) -> Self {
        Self::CompilerError(value)
    }
}

fn main() -> Result<(), RunCliError> {
    let Some(input_filename) = std::env::args().nth(1) else {
        eprintln!("Please provide a filename as the first argument.");
        return Err(RunCliError::MissingInput);
    };

    let source = std::io::read_to_string(
        File::open(input_filename.clone()).map_err(RunCliError::FileError)?,
    )
    .map_err(RunCliError::ReadWriteError)?;
    let mut compiler = Compiler::new(&source);
    let result = compiler.compile();
    match result {
        Ok(()) => {}
        Err(CompilerError::ParserError(ParserError::EndOfSource)) => {}
        Err(err) => return Err(err.into()),
    }

    let mut out = File::create(Path::new(&input_filename).with_extension("flb"))
        .map_err(RunCliError::FileError)?;
    write!(out, "{}", compiler.dump()).map_err(RunCliError::ReadWriteError)?;
    let mut interpreter = Vm::new(compiler.states);
    interpreter.interpret();
    let last_value = interpreter.stack().last().unwrap();
    println!("{last_value}");

    Ok(())
}
