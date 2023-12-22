use crate::state::Module;

pub mod fmt;
pub mod io;
pub mod iter;
pub mod string;

pub fn modules() -> Vec<Module> {
    vec![io::module(), iter::module(), string::module()]
}
