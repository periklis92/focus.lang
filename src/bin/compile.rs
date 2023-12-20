use std::{fs::File, path::Path};

use focus_third::compiler::{Compiler, CompilerError};

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
        std::io::read_to_string(File::open(&input_filename).map_err(CompileCliError::FileError)?)
            .map_err(CompileCliError::ReadWriteError)?;
    let mut compiler = Compiler::new(&source);
    // compiler.add_module(stdlib::string::module());
    // compiler.add_module(stdlib::io::module());
    // compiler.add_module(stdlib::iter::module());
    let module = compiler.compile_module("main")?;

    let mut out = File::create(Path::new(&input_filename).with_extension("flb"))
        .map_err(CompileCliError::FileError)?;
    module
        .dump(&mut out)
        .map_err(|e| CompileCliError::ReadWriteError(e))?;

    Ok(())
}
