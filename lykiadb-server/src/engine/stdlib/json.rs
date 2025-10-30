use crate::{
    engine::interpreter::{HaltReason, InterpretError, Interpreter},
    value::StdVal,
};
use serde_json::json;
use std::sync::Arc;

pub fn nt_json_encode(_interpreter: &mut Interpreter, args: &[StdVal]) -> Result<StdVal, HaltReason> {
    Ok(StdVal::Str(Arc::new(json!(args[0]).to_string())))
}

pub fn nt_json_decode(_interpreter: &mut Interpreter, args: &[StdVal]) -> Result<StdVal, HaltReason> {
    let json_str = match &args[0] {
        StdVal::Str(s) => s,
        _ => {
            return Err(HaltReason::Error(
                InterpretError::Other {
                    message: format!("json_decode: Unexpected argument '{:?}'", args[0]),
                }
                .into(),
            ));
        }
    };

    let parsed: StdVal = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            return Err(HaltReason::Error(
                InterpretError::Other {
                    message: format!("json_decode: Unhandled error '{e:?}'"),
                }
                .into(),
            ));
        }
    };

    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::interpreter::Output;
    use crate::util::alloc_shared;
    use rustc_hash::FxHashMap;
    use std::sync::Arc;

    fn setup() -> Interpreter {
        Interpreter::new(Some(alloc_shared(Output::new())), true)
    }

    #[test]
    fn test_json_encode() {
        let mut interpreter = setup();

        // Test primitive values
        assert_eq!(
            nt_json_encode(&mut interpreter, &[StdVal::Num(42.0)]).unwrap(),
            StdVal::Str(Arc::new("42.0".to_string()))
        );

        assert_eq!(
            nt_json_encode(&mut interpreter, &[StdVal::Bool(true)]).unwrap(),
            StdVal::Str(Arc::new("true".to_string()))
        );

        assert_eq!(
            nt_json_encode(&mut interpreter, &[StdVal::Str(Arc::new("hello".to_string()))]).unwrap(),
            StdVal::Str(Arc::new("\"hello\"".to_string()))
        );

        assert_eq!(
            nt_json_encode(&mut interpreter, &[StdVal::Undefined]).unwrap(),
            StdVal::Str(Arc::new("null".to_string()))
        );

        // Test array
        let arr = vec![StdVal::Num(1.0), StdVal::Str(Arc::new("test".to_string()))];
        let array_rv = StdVal::Array(alloc_shared(arr));

        assert_eq!(
            nt_json_encode(&mut interpreter, &[array_rv]).unwrap(),
            StdVal::Str(Arc::new("[1.0,\"test\"]".to_string()))
        );

        // Test object
        let mut map = FxHashMap::default();
        map.insert("key".to_string(), StdVal::Num(123.0));
        map.insert("msg".to_string(), StdVal::Str(Arc::new("value".to_string())));
        let object_rv = StdVal::Object(alloc_shared(map));

        assert_eq!(
            nt_json_encode(&mut interpreter, &[object_rv]).unwrap(),
            StdVal::Str(Arc::new("{\"key\":123.0,\"msg\":\"value\"}".to_string()))
        );
    }

    #[test]
    fn test_json_decode() {
        let mut interpreter = setup();

        // Test primitive values
        assert_eq!(
            nt_json_decode(&mut interpreter, &[StdVal::Str(Arc::new("42.0".to_string()))]).unwrap(),
            StdVal::Num(42.0)
        );

        assert_eq!(
            nt_json_decode(&mut interpreter, &[StdVal::Str(Arc::new("true".to_string()))]).unwrap(),
            StdVal::Bool(true)
        );

        assert_eq!(
            nt_json_decode(
                &mut interpreter,
                &[StdVal::Str(Arc::new("\"hello\"".to_string()))]
            )
            .unwrap(),
            StdVal::Str(Arc::new("hello".to_string()))
        );

        assert_eq!(
            nt_json_decode(&mut interpreter, &[StdVal::Str(Arc::new("null".to_string()))]).unwrap(),
            StdVal::Undefined
        );

        // Test array
        let array_result = nt_json_decode(
            &mut interpreter,
            &[StdVal::Str(Arc::new("[1.0, \"test\"]".to_string()))],
        )
        .unwrap();

        if let StdVal::Array(arr) = array_result {
            let arr = arr.read().unwrap();
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], StdVal::Num(1.0));
            assert_eq!(arr[1], StdVal::Str(Arc::new("test".to_string())));
        } else {
            panic!("Expected array result");
        }

        // Test object
        let object_result = nt_json_decode(
            &mut interpreter,
            &[StdVal::Str(Arc::new(
                "{\"key\": 123.0, \"msg\": \"value\"}".to_string(),
            ))],
        )
        .unwrap();

        if let StdVal::Object(obj) = object_result {
            let obj = obj.read().unwrap();
            assert_eq!(obj.len(), 2);
            assert_eq!(obj.get("key").unwrap(), &StdVal::Num(123.0));
            assert_eq!(
                obj.get("msg").unwrap(),
                &StdVal::Str(Arc::new("value".to_string()))
            );
        } else {
            panic!("Expected object result");
        }

        // Test error cases
        assert!(matches!(
            nt_json_decode(&mut interpreter, &[StdVal::Num(42.0)]),
            Err(HaltReason::Error(_))
        ));

        assert!(matches!(
            nt_json_decode(
                &mut interpreter,
                &[StdVal::Str(Arc::new("invalid json".to_string()))]
            ),
            Err(HaltReason::Error(_))
        ));
    }
}
