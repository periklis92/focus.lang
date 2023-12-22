use std::{fs::File, path::Path};

use focus_lang::{
    compiler::CompilerError,
    state::ModuleLoader,
    stdlib,
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

fn main() -> Result<(), RunCliError> {
    let Some(input_filename) = std::env::args().nth(1) else {
        eprintln!("Please provide a filename as the first argument.");
        return Err(RunCliError::MissingInput);
    };

    let mut module_loader = ModuleLoader::new("");
    module_loader.add_modules(stdlib::modules());
    module_loader.load_module(&input_filename);

    let mut out = File::create(Path::new(&input_filename).with_extension("flb"))
        .map_err(RunCliError::FileError)?;
    module_loader
        .module_at(4)
        .unwrap()
        .dump(&mut out)
        .map_err(|e| RunCliError::ReadWriteError(e))?;

    let module = module_loader
        .load_module_from_source("test", "let main () = Io.printf \"Hello World: {(2)}\"");
    let mut interpreter = Vm::new(module_loader);
    interpreter.execute_module(module, "main")?;
    let last_value = interpreter.stack().last().unwrap();
    println!("{last_value}");

    Ok(())
}
