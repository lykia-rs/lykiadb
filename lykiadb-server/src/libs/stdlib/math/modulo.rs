use lykiadb_lang::ast::Span;

use crate::{
    interpreter::{HaltReason, InterpretError, Interpreter},
    value::RV,
};

pub fn nt_modulo<'rv>(
    _interpreter: &mut Interpreter<'rv>,
    called_from: &Span,
    args: &[RV<'rv>],
) -> Result<RV<'rv>, HaltReason<'rv>> {
    let dividend = match &args[0] {
        RV::Double(n) => *n,
        _ => {
            return Err(HaltReason::Error(
                InterpretError::InvalidArgumentType {
                    span: *called_from,
                    expected: "number".to_string(),
                }
                .into(),
            ));
        }
    };

    let divisor = match &args[1] {
        RV::Double(n) if *n != 0.0 => *n,
        _ => {
            return Err(HaltReason::Error(
                InterpretError::InvalidArgumentType {
                    span: *called_from,
                    expected: "non-zero number".to_string(),
                }
                .into(),
            ));
        }
    };

    let result = dividend % divisor;
    Ok(RV::Double(result))
}

#[cfg(test)]
mod tests {
    use lykiadb_common::memory::alloc_shared;

    use super::*;
    use crate::interpreter::Output;
    use crate::interpreter::tests::create_test_interpreter;
    use std::sync::Arc;

    #[test]
    fn test_modulo_basic() {
        let mut interpreter = create_test_interpreter(Some(alloc_shared(Output::new())));
        let result = nt_modulo(
            &mut interpreter,
            &Span::default(),
            &[RV::Double(10.0), RV::Double(3.0)],
        );
        assert_eq!(result, Ok(RV::Double(1.0)));
    }

    #[test]
    fn test_modulo_negative() {
        let mut interpreter = create_test_interpreter(Some(alloc_shared(Output::new())));
        let result = nt_modulo(
            &mut interpreter,
            &Span::default(),
            &[RV::Double(-10.0), RV::Double(3.0)],
        );
        assert_eq!(result, Ok(RV::Double(-1.0)));
    }

    #[test]
    fn test_modulo_zero_dividend() {
        let mut interpreter = create_test_interpreter(Some(alloc_shared(Output::new())));
        let result = nt_modulo(
            &mut interpreter,
            &Span::default(),
            &[RV::Double(0.0), RV::Double(5.0)],
        );
        assert_eq!(result, Ok(RV::Double(0.0)));
    }

    #[test]
    fn test_modulo_zero_divisor() {
        let mut interpreter = create_test_interpreter(Some(alloc_shared(Output::new())));
        let result = nt_modulo(
            &mut interpreter,
            &Span::default(),
            &[RV::Double(10.0), RV::Double(0.0)],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_modulo_invalid_type() {
        let mut interpreter = create_test_interpreter(Some(alloc_shared(Output::new())));
        let result = nt_modulo(
            &mut interpreter,
            &Span::default(),
            &[RV::Str(Arc::new("foo".to_string())), RV::Double(3.0)],
        );
        assert!(result.is_err());
    }
}
