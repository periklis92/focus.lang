use crate::value::Value;

const STACK_MAX: usize = 64 * 256;

pub struct State {
    globals: Vec<Value>,
    stack: [Value; STACK_MAX],
}
