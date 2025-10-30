use crate::{
    engine::interpreter::{HaltReason, InterpretError, Interpreter},
    value::Value,
};

fn _calculate(n: f64) -> f64 {
    if n < 2. {
        return n;
    }
    _calculate(n - 1.) + _calculate(n - 2.)
}

pub fn nt_fib<V: Value>(_interpreter: &mut Interpreter<V>, args: &[V]) -> Result<V, HaltReason<V>> {
    if let Some(n) = args[0].as_number() {
        return Ok(V::number(_calculate(n)));
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

    type V = crate::value::StdVal;

    #[test]
    fn test_basic_fibonacci() {
        let mut interpreter = Interpreter::new(None, true);

        // Test first few Fibonacci numbers
        assert_eq!(
            nt_fib(&mut interpreter, &[V::number(0.0)]).unwrap(),
            V::number(0.0)
        );
        assert_eq!(
            nt_fib(&mut interpreter, &[V::number(1.0)]).unwrap(),
            V::number(1.0)
        );
        assert_eq!(
            nt_fib(&mut interpreter, &[V::number(2.0)]).unwrap(),
            V::number(1.0)
        );
        assert_eq!(
            nt_fib(&mut interpreter, &[V::number(3.0)]).unwrap(),
            V::number(2.0)
        );
        assert_eq!(
            nt_fib(&mut interpreter, &[V::number(4.0)]).unwrap(),
            V::number(3.0)
        );
        assert_eq!(
            nt_fib(&mut interpreter, &[V::number(5.0)]).unwrap(),
            V::number(5.0)
        );
        assert_eq!(
            nt_fib(&mut interpreter, &[V::number(6.0)]).unwrap(),
            V::number(8.0)
        );
        assert_eq!(
            nt_fib(&mut interpreter, &[V::number(7.0)]).unwrap(),
            V::number(13.0)
        );
    }

    #[test]
    fn test_invalid_input() {
        let mut interpreter = Interpreter::new(None, true);

        // Test with non-numeric input
        let result = nt_fib(&mut interpreter, &[V::boolean(true)]);
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
            nt_fib(&mut interpreter, &[V::number(-1.0)]).unwrap(),
            V::number(-1.0)
        );
        assert_eq!(
            nt_fib(&mut interpreter, &[V::number(-5.0)]).unwrap(),
            V::number(-5.0)
        );
    }
}
