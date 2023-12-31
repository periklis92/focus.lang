use std::{fs::File, path::Path};

use focus_lang::{
    compiler::CompilerError,
    value::Value,
    vm::{RuntimeError, Vm},
};

#[derive(Debug)]
enum RunCliError {
    MissingInput,
    ReadWriteError(std::io::Error),
    FileError(std::io::Error),
    CompilerError(CompilerError),
    RuntimeError(RuntimeError),
}

impl From<CompilerError> for RunCliError {
    fn from(value: CompilerError) -> Self {
        Self::CompilerError(value)
    }
}

impl From<RuntimeError> for RunCliError {
    fn from(value: RuntimeError) -> Self {
        Self::RuntimeError(value)
    }
}

fn main() -> Result<Value, RunCliError> {
    let Some(input_filename) = std::env::args().nth(1) else {
        eprintln!("Please provide a filename as the first argument.");
        return Err(RunCliError::MissingInput);
    };

    let source = std::fs::read_to_string(&input_filename).map_err(RunCliError::ReadWriteError)?;

    let mut out = File::create(Path::new(&input_filename).with_extension("flb"))
        .map_err(RunCliError::FileError)?;

    let mut vm = Vm::new_with_std();
    let result = vm.load_from_source("main", &source)?;

    vm.module_loader()
        .module_at(result)
        .unwrap()
        .dump(&mut out)
        .map_err(RunCliError::ReadWriteError)?;

    match vm.execute_module(result, "main") {
        Ok(_) => {}
        Err(err) => {
            println!("There was an error: {err}");
            println!("{}", vm.stack_trace(5));
            return Err(err.into());
        }
    }

    let last_value = vm.stack().last().unwrap();

    Ok(last_value.clone())
}
