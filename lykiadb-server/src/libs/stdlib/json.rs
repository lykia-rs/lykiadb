use crate::{
    engine::interpreter::{HaltReason, InterpretError, Interpreter}, lykia_lambda, lykia_module, value::RV
};
use lykiadb_lang::ast::Span;
use serde_json::json;
use std::sync::Arc;


pub fn nt_json_encode(
    _interpreter: &mut Interpreter,
    called_from: &Span,
    args: &[RV],
) -> Result<RV, HaltReason> {
    Ok(RV::Str(Arc::new(json!(args[0]).to_string())))
}

pub fn nt_json_decode(
    _interpreter: &mut Interpreter,
    called_from: &Span,
    args: &[RV],
) -> Result<RV, HaltReason> {
    let json_str = match &args[0] {
        RV::Str(s) => s,
        _ => {
            return Err(HaltReason::Error(
                InterpretError::InvalidArgumentType {
                    span: *called_from,
                    expected: "string".to_string(),
                }
                .into(),
            ));
        }
    };

    let parsed: RV = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            return Err(HaltReason::Error(
                InterpretError::InvalidArgumentType {
                    span: *called_from,
                    expected: "JSON".to_string(),
                }
                .into(),
            ));
        }
    };

    Ok(parsed)
}

lykia_module!(json, {
    stringify => lykia_lambda!(nt_json_encode),
    parse => lykia_lambda!(nt_json_decode)
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::interpreter::Output;
    use crate::engine::interpreter::tests::create_test_interpreter;
    use crate::util::alloc_shared;
    use crate::value::{array::RVArray, object::RVObject};
    use rustc_hash::FxHashMap;
    use std::sync::Arc;

    #[test]
    fn test_json_encode() {
        let mut interpreter = create_test_interpreter(Some(alloc_shared(Output::new())));

        // Test primitive values
        assert_eq!(
            nt_json_encode(&mut interpreter, &Span::default(), &[RV::Num(42.0)]).unwrap(),
            RV::Str(Arc::new("42.0".to_string()))
        );

        assert_eq!(
            nt_json_encode(&mut interpreter, &Span::default(), &[RV::Bool(true)]).unwrap(),
            RV::Str(Arc::new("true".to_string()))
        );

        assert_eq!(
            nt_json_encode(
                &mut interpreter,
                &Span::default(),
                &[RV::Str(Arc::new("hello".to_string()))]
            )
            .unwrap(),
            RV::Str(Arc::new("\"hello\"".to_string()))
        );

        assert_eq!(
            nt_json_encode(&mut interpreter, &Span::default(), &[RV::Undefined]).unwrap(),
            RV::Str(Arc::new("null".to_string()))
        );

        // Test array
        let arr = vec![RV::Num(1.0), RV::Str(Arc::new("test".to_string()))];
        let array_rv = RV::Array(RVArray::from_vec(arr));

        assert_eq!(
            nt_json_encode(&mut interpreter, &Span::default(), &[array_rv]).unwrap(),
            RV::Str(Arc::new("[1.0,\"test\"]".to_string()))
        );

        // Test object
        let mut map = FxHashMap::default();
        map.insert("key".to_string(), RV::Num(123.0));
        map.insert("msg".to_string(), RV::Str(Arc::new("value".to_string())));
        let object_rv = RV::Object(RVObject::from_map(map));

        assert_eq!(
            nt_json_encode(&mut interpreter, &Span::default(), &[object_rv]).unwrap(),
            RV::Str(Arc::new("{\"key\":123.0,\"msg\":\"value\"}".to_string()))
        );
    }

    #[test]
    fn test_json_decode() {
        let mut interpreter = create_test_interpreter(Some(alloc_shared(Output::new())));

        // Test primitive values
        assert_eq!(
            nt_json_decode(
                &mut interpreter,
                &Span::default(),
                &[RV::Str(Arc::new("42.0".to_string()))]
            )
            .unwrap(),
            RV::Num(42.0)
        );

        assert_eq!(
            nt_json_decode(
                &mut interpreter,
                &Span::default(),
                &[RV::Str(Arc::new("true".to_string()))]
            )
            .unwrap(),
            RV::Bool(true)
        );

        assert_eq!(
            nt_json_decode(
                &mut interpreter,
                &Span::default(),
                &[RV::Str(Arc::new("\"hello\"".to_string()))]
            )
            .unwrap(),
            RV::Str(Arc::new("hello".to_string()))
        );

        assert_eq!(
            nt_json_decode(
                &mut interpreter,
                &Span::default(),
                &[RV::Str(Arc::new("null".to_string()))]
            )
            .unwrap(),
            RV::Undefined
        );

        // Test array
        let array_result = nt_json_decode(
            &mut interpreter,
            &Span::default(),
            &[RV::Str(Arc::new("[1.0, \"test\"]".to_string()))],
        )
        .unwrap();

        if let RV::Array(arr) = array_result {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr.get(0), RV::Num(1.0));
            assert_eq!(arr.get(1), RV::Str(Arc::new("test".to_string())));
        } else {
            panic!("Expected array result");
        }

        // Test object
        let object_result = nt_json_decode(
            &mut interpreter,
            &Span::default(),
            &[RV::Str(Arc::new(
                "{\"key\": 123.0, \"msg\": \"value\"}".to_string(),
            ))],
        )
        .unwrap();

        if let RV::Object(obj) = object_result {
            assert_eq!(obj.len(), 2);
            assert_eq!(obj.get("key").unwrap(), RV::Num(123.0));
            assert_eq!(
                obj.get("msg").unwrap(),
                RV::Str(Arc::new("value".to_string()))
            );
        } else {
            panic!("Expected object result");
        }

        // Test error cases
        assert!(matches!(
            nt_json_decode(&mut interpreter, &Span::default(), &[RV::Num(42.0)]),
            Err(HaltReason::Error(_))
        ));

        assert!(matches!(
            nt_json_decode(
                &mut interpreter,
                &Span::default(),
                &[RV::Str(Arc::new("invalid json".to_string()))]
            ),
            Err(HaltReason::Error(_))
        ));
    }
}
