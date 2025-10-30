use crate::{
    engine::interpreter::{HaltReason, Interpreter},
    value::Value,
};

pub fn nt_print<V: Value>(_interpreter: &mut Interpreter<V>, args: &[V]) -> Result<V, HaltReason<V>> {
    for arg in args {
        print!("{arg:?} ");
    }
    println!();
    Ok(V::undefined())
}
