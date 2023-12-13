use std::{fs::File, io::Write};

use focus_third::{
    compiler::{Compiler, CompilerError},
    parser::ParserError,
};

#[derive(Debug)]
enum CompileCliError {
    MissingInput,
    ReadWriteError(std::io::Error),
    FileError(std::io::Error),
    CompilerError(CompilerError),
}

impl From<CompilerError> for CompileCliError {
    fn from(value: CompilerError) -> Self {
        Self::CompilerError(value)
    }
}

fn main() -> Result<(), CompileCliError> {
    let Some(input_filename) = std::env::args().nth(1) else {
        eprintln!("Please provide a filename as the first argument.");
        return Err(CompileCliError::MissingInput);
    };

    let source =
        std::io::read_to_string(File::open(input_filename).map_err(CompileCliError::FileError)?)
            .map_err(CompileCliError::ReadWriteError)?;
    let mut compiler = Compiler::new(&source);
    let result = compiler.compile();
    match result {
        Ok(()) => {}
        Err(CompilerError::ParserError(ParserError::EndOfSource)) => {}
        Err(err) => return Err(err.into()),
    }
    let mut out = File::create("out.flb").map_err(CompileCliError::FileError)?;
    write!(out, "{}", compiler.dump()).map_err(CompileCliError::ReadWriteError)?;
    /*out.write_fmt(format_args!("{}", compiler.proto()))
    .map_err(CompileCliError::ReadWriteError)?;*/

    Ok(())
}
