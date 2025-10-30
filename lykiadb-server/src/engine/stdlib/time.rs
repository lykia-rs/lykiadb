use crate::engine::interpreter::{HaltReason, Interpreter};
use crate::value::Value;
use std::time;

pub fn nt_clock<V: Value>(_interpreter: &mut Interpreter<V>, _args: &[V]) -> Result<V, HaltReason<V>> {
    if let Ok(n) = time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
        return Ok(V::number(n.as_secs_f64()));
    }
    Ok(V::undefined())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::interpreter::Output;
    use crate::util::alloc_shared;
    use crate::value::StdVal;

    fn setup<V: Value>() -> Interpreter<V> {
        Interpreter::new(Some(alloc_shared(Output::new())), true)
    }

    #[test]
    fn test_nt_clock() {
        let mut interpreter = setup::<StdVal>();

        // Test clock function
        let result = nt_clock(&mut interpreter, &[]);
        assert!(result.is_ok());

        let clock = result.unwrap();
        
        if clock.as_number().is_none() {
            panic!("Expected number result from clock function");
        }
    }
}
