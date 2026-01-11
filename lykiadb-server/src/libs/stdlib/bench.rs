use lykiadb_lang::ast::Span;

use crate::{
    engine::interpreter::{HaltReason, InterpretError, Interpreter},
    lykia_module, lykia_native_fn,
    value::RV,
};

fn _calculate(n: f64) -> f64 {
    if n < 2. {
        return n;
    }
    _calculate(n - 1.) + _calculate(n - 2.)
}

pub fn nt_fib(
    _interpreter: &mut Interpreter,
    called_from: &Span,
    args: &[RV],
) -> Result<RV, HaltReason> {
    if let RV::Num(n) = args[0] {
        return Ok(RV::Num(_calculate(n)));
    }
    Err(HaltReason::Error(
        InterpretError::InvalidArgumentType {
            span: *called_from,
            expected: "number".to_string(),
        }
        .into(),
    ))
}

lykia_module!(bench, {
    fib => lykia_native_fn!(nt_fib)
}, {}, []);

#[cfg(test)]
mod tests {
    use lykiadb_common::extract;

    use crate::engine::{error::ExecutionError, interpreter::tests::create_test_interpreter};

    use super::*;

    #[test]
    fn test_basic_fibonacci() {
        let mut interpreter = create_test_interpreter(None);

        // Test first few Fibonacci numbers
        assert_eq!(
            nt_fib(&mut interpreter, &Span::default(), &[RV::Num(0.0)]),
            Ok(RV::Num(0.0))
        );
        assert_eq!(
            nt_fib(&mut interpreter, &Span::default(), &[RV::Num(1.0)]),
            Ok(RV::Num(1.0))
        );
        assert_eq!(
            nt_fib(&mut interpreter, &Span::default(), &[RV::Num(2.0)]),
            Ok(RV::Num(1.0))
        );
        assert_eq!(
            nt_fib(&mut interpreter, &Span::default(), &[RV::Num(3.0)]),
            Ok(RV::Num(2.0))
        );
        assert_eq!(
            nt_fib(&mut interpreter, &Span::default(), &[RV::Num(4.0)]),
            Ok(RV::Num(3.0))
        );
        assert_eq!(
            nt_fib(&mut interpreter, &Span::default(), &[RV::Num(5.0)]),
            Ok(RV::Num(5.0))
        );
        assert_eq!(
            nt_fib(&mut interpreter, &Span::default(), &[RV::Num(6.0)]),
            Ok(RV::Num(8.0))
        );
        assert_eq!(
            nt_fib(&mut interpreter, &Span::default(), &[RV::Num(7.0)]),
            Ok(RV::Num(13.0))
        );
    }

    #[test]
    fn test_invalid_input() {
        let mut interpreter = create_test_interpreter(None);

        // Test with non-numeric input
        let result = nt_fib(&mut interpreter, &Span::default(), &[RV::Bool(true)]);
        assert!(result.is_err());

        let err = result.unwrap_err();
        extract!(HaltReason::Error(
            ExecutionError::Interpret(
                InterpretError::InvalidArgumentType {
                    expected,
                    ..
                }
            )
        ), err);
        
        assert_eq!(expected, "number");
    }

    #[test]
    fn test_negative_input() {
        let mut interpreter = create_test_interpreter(None);

        // Negative numbers should return themselves as per implementation
        assert_eq!(
            nt_fib(&mut interpreter, &Span::default(), &[RV::Num(-1.0)]),
            Ok(RV::Num(-1.0))
        );
        assert_eq!(
            nt_fib(&mut interpreter, &Span::default(), &[RV::Num(-5.0)]),
            Ok(RV::Num(-5.0))
        );
    }
}
