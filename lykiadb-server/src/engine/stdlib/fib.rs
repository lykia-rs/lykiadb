use crate::{
    engine::interpreter::{HaltReason, InterpretError, Interpreter},
    value::StdVal,
};

fn _calculate(n: f64) -> f64 {
    if n < 2. {
        return n;
    }
    _calculate(n - 1.) + _calculate(n - 2.)
}

pub fn nt_fib(_interpreter: &mut Interpreter, args: &[StdVal]) -> Result<StdVal, HaltReason> {
    if let StdVal::Num(n) = args[0] {
        return Ok(StdVal::Num(_calculate(n)));
    }
    Err(HaltReason::Error(
        InterpretError::Other {
            message: format!("fib_nat: Unexpected argument '{:?}'", args[0]),
        }
        .into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_fibonacci() {
        let mut interpreter = Interpreter::new(None, true);

        // Test first few Fibonacci numbers
        assert_eq!(
            nt_fib(&mut interpreter, &[StdVal::Num(0.0)]).unwrap(),
            StdVal::Num(0.0)
        );
        assert_eq!(
            nt_fib(&mut interpreter, &[StdVal::Num(1.0)]).unwrap(),
            StdVal::Num(1.0)
        );
        assert_eq!(
            nt_fib(&mut interpreter, &[StdVal::Num(2.0)]).unwrap(),
            StdVal::Num(1.0)
        );
        assert_eq!(
            nt_fib(&mut interpreter, &[StdVal::Num(3.0)]).unwrap(),
            StdVal::Num(2.0)
        );
        assert_eq!(
            nt_fib(&mut interpreter, &[StdVal::Num(4.0)]).unwrap(),
            StdVal::Num(3.0)
        );
        assert_eq!(
            nt_fib(&mut interpreter, &[StdVal::Num(5.0)]).unwrap(),
            StdVal::Num(5.0)
        );
        assert_eq!(
            nt_fib(&mut interpreter, &[StdVal::Num(6.0)]).unwrap(),
            StdVal::Num(8.0)
        );
        assert_eq!(
            nt_fib(&mut interpreter, &[StdVal::Num(7.0)]).unwrap(),
            StdVal::Num(13.0)
        );
    }

    #[test]
    fn test_invalid_input() {
        let mut interpreter = Interpreter::new(None, true);

        // Test with non-numeric input
        let result = nt_fib(&mut interpreter, &[StdVal::Bool(true)]);
        assert!(result.is_err());

        let err = result.unwrap_err();
        match err {
            HaltReason::Error(e) => {
                assert!(e.to_string().contains("Unexpected argument"));
            }
            _ => panic!("Expected InterpretError"),
        }
    }

    #[test]
    fn test_negative_input() {
        let mut interpreter = Interpreter::new(None, true);

        // Negative numbers should return themselves as per implementation
        assert_eq!(
            nt_fib(&mut interpreter, &[StdVal::Num(-1.0)]).unwrap(),
            StdVal::Num(-1.0)
        );
        assert_eq!(
            nt_fib(&mut interpreter, &[StdVal::Num(-5.0)]).unwrap(),
            StdVal::Num(-5.0)
        );
    }
}
