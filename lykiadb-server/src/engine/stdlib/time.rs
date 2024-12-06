use crate::engine::interpreter::{HaltReason, Interpreter};
use crate::value::RV;
use std::time;

pub fn nt_clock(_interpreter: &mut Interpreter, _args: &[RV]) -> Result<RV, HaltReason> {
    if let Ok(n) = time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
        return Ok(RV::Num(n.as_secs_f64()));
    }
    Ok(RV::Undefined)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::interpreter::Output;
    use crate::util::alloc_shared;

    fn setup() -> Interpreter {
        Interpreter::new(Some(alloc_shared(Output::new())), true)
    }

    #[test]
    fn test_nt_clock() {
        let mut interpreter = setup();

        // Test clock function
        let result = nt_clock(&mut interpreter, &[]);
        assert!(result.is_ok());

        let clock = result.unwrap();
        if let RV::Num(_) = clock {
            // Clock function returns a number
        } else {
            panic!("Expected number result from clock function");
        }
    }
}
