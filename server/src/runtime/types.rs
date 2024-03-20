use super::environment::EnvId;
use crate::lang::ast::expr::Operation;
use crate::lang::ast::program::Program;
use crate::lang::ast::stmt::StmtId;
use crate::runtime::interpreter::{HaltReason, Interpreter};
use crate::util::{alloc_shared, Shared};
use rustc_hash::FxHashMap;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter};
use std::ops;
use std::sync::{Arc, RwLock};

pub trait Stateful {
    fn call(&mut self, interpreter: &mut Interpreter, rv: &[RV]) -> Result<RV, HaltReason>;
}

#[derive(Clone)]
pub enum Function {
    Lambda {
        function: fn(&mut Interpreter, &[RV]) -> Result<RV, HaltReason>,
    },
    Stateful(Shared<dyn Stateful + Send + Sync>),
    UserDefined {
        name: String,
        program: Arc<Program>,
        parameters: Vec<String>,
        closure: EnvId,
        body: Arc<Vec<StmtId>>,
    },
}

impl Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Function::Stateful(_) | Function::Lambda { function: _ } => write!(f, "<native_fn>"),
            Function::UserDefined {
                name,
                program: _,
                parameters: _,
                closure: _,
                body: _,
            } => write!(f, "{}", name),
        }
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}

impl Function {
    pub fn call(&self, interpreter: &mut Interpreter, arguments: &[RV]) -> Result<RV, HaltReason> {
        match self {
            Function::Stateful(stateful) => stateful.write().unwrap().call(interpreter, arguments),
            Function::Lambda { function } => function(interpreter, arguments),
            Function::UserDefined {
                name,
                program,
                parameters,
                closure,
                body,
            } => {
                interpreter.user_fn_call(body, *closure, parameters, arguments)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum RV {
    Str(Arc<String>),
    Num(f64),
    Bool(bool),
    Object(Shared<FxHashMap<String, RV>>),
    Array(Shared<Vec<RV>>),
    Callable(Option<usize>, Arc<Function>),
    Undefined,
    NaN,
    Null,
}

impl<'de> Deserialize<'de> for RV {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::String(s) => Ok(RV::Str(Arc::new(s))),
            serde_json::Value::Number(n) => Ok(RV::Num(n.as_f64().unwrap())),
            serde_json::Value::Bool(b) => Ok(RV::Bool(b)),
            serde_json::Value::Array(arr) => {
                let mut vec = Vec::new();
                for item in arr {
                    vec.push(serde_json::from_value(item).unwrap());
                }
                Ok(RV::Array(alloc_shared(vec)))
            }
            serde_json::Value::Object(obj) => {
                let mut map = FxHashMap::default();
                for (key, value) in obj {
                    map.insert(key, serde_json::from_value(value).unwrap());
                }
                Ok(RV::Object(alloc_shared(map)))
            }
            serde_json::Value::Null => Ok(RV::Null),
        }
    }
}

impl Serialize for RV {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            RV::Str(s) => serializer.serialize_str(s),
            RV::Num(n) => serializer.serialize_f64(*n),
            RV::Bool(b) => serializer.serialize_bool(*b),
            RV::Undefined => serializer.serialize_none(),
            RV::NaN => serializer.serialize_none(),
            RV::Null => serializer.serialize_none(),
            RV::Callable(_, _) => serializer.serialize_none(),
            RV::Array(arr) => {
                let mut seq = serializer.serialize_seq(None).unwrap();
                let arr = (arr.borrow() as &RwLock<Vec<RV>>).read().unwrap();
                for item in (&arr).iter() {
                    seq.serialize_element(&item)?;
                }
                seq.end()
            }
            RV::Object(obj) => {
                let mut map = serializer.serialize_map(None).unwrap();
                let arr = (obj.borrow() as &RwLock<FxHashMap<String, RV>>)
                    .read()
                    .unwrap();
                for (key, value) in (&arr).iter() {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
        }
    }
}

impl RV {
    pub fn is_truthy(&self) -> bool {
        match &self {
            RV::Num(value) => !value.is_nan() && value.abs() > 0.0,
            RV::Str(value) => !value.is_empty(),
            RV::Bool(value) => *value,
            RV::Null | RV::Undefined | RV::NaN => false,
            _ => true,
        }
    }

    pub fn eq_any_bool(&self, b: bool) -> bool {
        self.is_truthy() == b
    }

    pub fn eq_str_num(&self, n: f64) -> bool {
        if let RV::Str(s) = self {
            if let Ok(num) = s.parse::<f64>() {
                return num == n;
            }
        }
        false
    }

    pub fn partial_cmp_str_bool(&self, other: bool) -> Option<std::cmp::Ordering> {
        if let Some(num) = self.as_number() {
            return num.partial_cmp(&if other { 1.0 } else { 0.0 });
        }
        self.is_truthy().partial_cmp(&other)
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            RV::Num(value) => Some(*value),
            RV::Bool(true) => Some(1.0),
            RV::Bool(false) => Some(0.0),
            RV::Str(s) => {
                if let Ok(num) = s.parse::<f64>() {
                    Some(num)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl PartialEq for RV {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RV::Array(_), RV::Array(_)) | (RV::Object(_), RV::Object(_)) => false,
            //
            (RV::Callable(_, _), RV::Callable(_, _)) => false,
            //
            (RV::Null, RV::Null) => true,
            (RV::Undefined, RV::Undefined) => true,
            (RV::NaN, RV::NaN) => true,
            (RV::Null, RV::Undefined) => true,
            (RV::Undefined, RV::Null) => true,
            //
            (RV::NaN, _) | (_, RV::NaN) => false,
            (RV::Null, _) | (_, RV::Null) => false,
            (RV::Undefined, _) | (_, RV::Undefined) => false,
            //
            (RV::Str(a), RV::Str(b)) => a == b,
            (RV::Num(a), RV::Num(b)) => a == b,
            (RV::Bool(a), RV::Bool(b)) => a == b,
            //
            (RV::Str(_), RV::Num(b)) => self.eq_str_num(*b),
            (RV::Num(a), RV::Str(_)) => other.eq_str_num(*a),
            //
            (RV::Str(_), RV::Bool(b)) => self.eq_any_bool(*b),
            (RV::Bool(a), RV::Str(_)) => other.eq_any_bool(*a),
            //
            (RV::Num(_), RV::Bool(b)) => self.eq_any_bool(*b),
            (RV::Bool(a), RV::Num(_)) => other.eq_any_bool(*a),
            //
            _ => false,
        }
    }
}

impl PartialOrd for RV {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (RV::Array(_), RV::Array(_)) | (RV::Object(_), RV::Object(_)) => None,
            //
            (RV::Callable(_, _), RV::Callable(_, _)) => None,
            //
            (RV::Null, RV::Null) => Some(std::cmp::Ordering::Equal),
            (RV::Undefined, RV::Undefined) => Some(std::cmp::Ordering::Equal),
            (RV::NaN, RV::NaN) => Some(std::cmp::Ordering::Equal),
            (RV::Null, RV::Undefined) => Some(std::cmp::Ordering::Equal),
            (RV::Undefined, RV::Null) => Some(std::cmp::Ordering::Equal),
            //
            (RV::NaN, _) | (_, RV::NaN) => None,
            (RV::Null, _) | (_, RV::Null) => None,
            (RV::Undefined, _) | (_, RV::Undefined) => None,
            //
            (RV::Str(a), RV::Str(b)) => Some(a.cmp(b)),
            (RV::Num(a), RV::Num(b)) => a.partial_cmp(b),
            (RV::Bool(a), RV::Bool(b)) => a.partial_cmp(b),
            //
            (RV::Str(a), RV::Num(b)) => {
                if let Ok(num) = a.parse::<f64>() {
                    return num.partial_cmp(b);
                }
                None
            }
            (RV::Num(a), RV::Str(b)) => {
                if let Ok(num) = b.parse::<f64>() {
                    return a.partial_cmp(&num);
                }
                None
            }
            //
            (RV::Str(_), RV::Bool(b)) => self.partial_cmp_str_bool(*b),
            (RV::Bool(a), RV::Str(_)) => other.partial_cmp_str_bool(*a),
            //
            (RV::Num(num), RV::Bool(b)) => num.partial_cmp(&if *b { 1.0 } else { 0.0 }),
            (RV::Bool(b), RV::Num(num)) => (if *b { 1.0 } else { 0.0 }).partial_cmp(num),
            //
            _ => None,
        }
    }
}

impl ops::Add for RV {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (RV::NaN, _) | (_, RV::NaN) => RV::NaN,
            //
            (RV::Bool(_), RV::Bool(_)) | (RV::Num(_), RV::Bool(_)) | (RV::Bool(_), RV::Num(_)) => {
                RV::Num(self.as_number().unwrap() + rhs.as_number().unwrap())
            }

            (RV::Num(l), RV::Num(r)) => RV::Num(l + r),
            //
            (RV::Str(l), RV::Str(r)) => RV::Str(Arc::new(l.to_string() + &r.to_string())),
            //
            (RV::Str(s), RV::Num(num)) => RV::Str(Arc::new(s.to_string() + &num.to_string())),
            (RV::Num(num), RV::Str(s)) => RV::Str(Arc::new(num.to_string() + &s.to_string())),
            //
            (RV::Str(s), RV::Bool(bool)) => RV::Str(Arc::new(s.to_string() + &bool.to_string())),
            (RV::Bool(bool), RV::Str(s)) => RV::Str(Arc::new(bool.to_string() + &s.to_string())),
            //
            (_, _) => RV::NaN,
        }
    }
}

impl ops::Sub for RV {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (RV::Undefined, _) | (_, RV::Undefined) => RV::NaN,
            (RV::NaN, _) | (_, RV::NaN) => RV::NaN,
            (RV::Null, _) | (_, RV::Null) => RV::Num(0.0),
            (l, r) => l
                .as_number()
                .and_then(|a| r.as_number().map(|b| (a, b)))
                .map(|(a, b)| RV::Num(a - b))
                .unwrap_or(RV::NaN),
        }
    }
}

impl ops::Mul for RV {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (RV::Undefined, _) | (_, RV::Undefined) => RV::NaN,
            (RV::NaN, _) | (_, RV::NaN) => RV::NaN,
            (RV::Null, _) | (_, RV::Null) => RV::Num(0.0),
            (l, r) => l
                .as_number()
                .and_then(|a| r.as_number().map(|b| (a, b)))
                .map(|(a, b)| RV::Num(a * b))
                .unwrap_or(RV::NaN),
        }
    }
}

impl ops::Div for RV {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (RV::Undefined, _) | (_, RV::Undefined) => RV::NaN,
            (RV::NaN, _) | (_, RV::NaN) => RV::NaN,
            (RV::Null, _) | (_, RV::Null) => RV::Num(0.0),
            (l, r) => l
                .as_number()
                .and_then(|a| r.as_number().map(|b| (a, b)))
                .map(|(a, b)| {
                    if a == 0.0 && b == 0.0 {
                        RV::NaN
                    } else {
                        RV::Num(a / b)
                    }
                })
                .unwrap_or(RV::NaN),
        }
    }
}

#[inline(always)]
pub fn eval_binary(left_eval: RV, right_eval: RV, operation: Operation) -> RV {
    /*
        TODO(vck):
            - Add support for object operations
            - Add support for array operations
            - Add support for function operations
    */
    match operation {
        Operation::IsEqual => RV::Bool(left_eval == right_eval),
        Operation::IsNotEqual => RV::Bool(left_eval != right_eval),
        Operation::Less => RV::Bool(left_eval < right_eval),
        Operation::LessEqual => RV::Bool(left_eval <= right_eval),
        Operation::Greater => RV::Bool(left_eval > right_eval),
        Operation::GreaterEqual => RV::Bool(left_eval >= right_eval),
        Operation::Add => left_eval + right_eval,
        Operation::Subtract => left_eval - right_eval,
        Operation::Multiply => left_eval * right_eval,
        Operation::Divide => left_eval / right_eval,
        _ => RV::Undefined,
    }
}

#[cfg(test)]
mod test {
    use std::{f64::INFINITY, sync::Arc};

    use rustc_hash::FxHashMap;

    use crate::{
        lang::ast::expr::Operation,
        runtime::types::{eval_binary, Function, RV},
        util::alloc_shared,
    };

    #[test]
    fn test_is_value_truthy() {
        assert_eq!((RV::Null).is_truthy(), false);
        assert_eq!((RV::Undefined).is_truthy(), false);
        assert_eq!((RV::NaN).is_truthy(), false);
        assert_eq!((RV::Bool(false)).is_truthy(), false);
        assert_eq!((RV::Bool(true)).is_truthy(), true);
        assert_eq!((RV::Num(0.0)).is_truthy(), false);
        assert_eq!((RV::Num(0.1)).is_truthy(), true);
        assert_eq!((RV::Num(-0.1)).is_truthy(), true);
        assert_eq!((RV::Num(1.0)).is_truthy(), true);
        assert_eq!((RV::Num(-1.0)).is_truthy(), true);
        assert_eq!((RV::Str(Arc::new("".to_owned()))).is_truthy(), false);
        assert_eq!((RV::Str(Arc::new("0".to_owned()))).is_truthy(), true);
        assert_eq!((RV::Str(Arc::new("false".to_owned()))).is_truthy(), true);
        assert_eq!((RV::Str(Arc::new("true".to_owned()))).is_truthy(), true);
        assert_eq!((RV::Str(Arc::new("foo".to_owned()))).is_truthy(), true);
        assert_eq!((RV::Array(alloc_shared(vec![]))).is_truthy(), true);
        assert_eq!(
            (RV::Object(alloc_shared(FxHashMap::default()))).is_truthy(),
            true
        );
        assert_eq!(
            (RV::Callable(
                Some(1),
                Arc::new(Function::Lambda {
                    function: |_, _| Ok(RV::Undefined)
                })
            ))
            .is_truthy(),
            true
        );
    }

    #[test]
    fn test_as_number() {
        assert_eq!((RV::Num(1.0)).as_number(), Some(1.0));
        assert_eq!((RV::Bool(false)).as_number(), Some(0.0));
        assert_eq!((RV::Bool(true)).as_number(), Some(1.0));
        assert_eq!((RV::Str(Arc::new("".to_owned()))).as_number(), None);
    }

    #[test]
    fn test_eval_binary_addition() {
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(2.0), Operation::Add),
            RV::Num(3.0)
        );
        assert_eq!(
            eval_binary(RV::Num(2.0), RV::Num(1.0), Operation::Add),
            RV::Num(3.0)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("b".to_string())),
                Operation::Add
            ),
            RV::Str(Arc::new("ab".to_string()))
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("b".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::Add
            ),
            RV::Str(Arc::new("ba".to_string()))
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(true), Operation::Add),
            RV::Num(2.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(false), Operation::Add),
            RV::Num(1.0)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(true), Operation::Add),
            RV::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(false), Operation::Add),
            RV::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::Add),
            RV::Num(2.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Add),
            RV::Num(2.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Num(1.0), Operation::Add),
            RV::Num(1.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(false), Operation::Add),
            RV::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Add),
            RV::Num(2.0)
        );
    }

    #[test]
    fn test_eval_binary_subtraction() {
        assert_eq!(
            eval_binary(RV::Num(2.0), RV::Num(1.0), Operation::Subtract),
            RV::Num(1.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(2.0), Operation::Subtract),
            RV::Num(-1.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(true), Operation::Subtract),
            RV::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(false), Operation::Subtract),
            RV::Num(1.0)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(true), Operation::Subtract),
            RV::Num(-1.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(false), Operation::Subtract),
            RV::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::Subtract),
            RV::Num(0.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Subtract),
            RV::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Num(1.0), Operation::Subtract),
            RV::Num(-1.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(false), Operation::Subtract),
            RV::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Subtract),
            RV::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("b".to_string())),
                Operation::Subtract
            ),
            RV::NaN
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("b".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::Subtract
            ),
            RV::NaN
        );
    }

    #[test]
    fn test_eval_binary_multiplication() {
        assert_eq!(
            eval_binary(RV::Num(2.0), RV::Num(1.0), Operation::Multiply),
            RV::Num(2.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(2.0), Operation::Multiply),
            RV::Num(2.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(true), Operation::Multiply),
            RV::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(false), Operation::Multiply),
            RV::Num(0.0)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(true), Operation::Multiply),
            RV::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(false), Operation::Multiply),
            RV::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::Multiply),
            RV::Num(1.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Multiply),
            RV::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Num(1.0), Operation::Multiply),
            RV::Num(0.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(false), Operation::Multiply),
            RV::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Multiply),
            RV::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("b".to_string())),
                Operation::Multiply
            ),
            RV::NaN
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("b".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::Multiply
            ),
            RV::NaN
        );
    }

    #[test]
    fn test_eval_binary_division() {
        assert_eq!(
            eval_binary(RV::Num(2.0), RV::Num(1.0), Operation::Divide),
            RV::Num(2.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(2.0), Operation::Divide),
            RV::Num(0.5)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(true), Operation::Divide),
            RV::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(false), Operation::Divide),
            RV::Num(INFINITY)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(true), Operation::Divide),
            RV::Num(0.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(false), Operation::Divide),
            RV::NaN
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::Divide),
            RV::Num(1.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Divide),
            RV::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Num(1.0), Operation::Divide),
            RV::Num(0.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(false), Operation::Divide),
            RV::Num(INFINITY)
        );
        //
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Divide),
            RV::Num(1.0)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("b".to_string())),
                Operation::Divide
            ),
            RV::NaN
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("b".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::Divide
            ),
            RV::NaN
        );
    }

    #[test]
    fn test_eval_binary_is_equal() {
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(2.0), Operation::IsEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(1.0), Operation::IsEqual),
            RV::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(true), Operation::IsEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(false), Operation::IsEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(true), Operation::IsEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(false), Operation::IsEqual),
            RV::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::IsEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::IsEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Num(1.0), Operation::IsEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(false), Operation::IsEqual),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Num(1.0),
                RV::Str(Arc::new("a".to_string())),
                Operation::IsEqual
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Num(1.0),
                Operation::IsEqual
            ),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("b".to_string())),
                Operation::IsEqual
            ),
            RV::Bool(false)
        );
    }

    #[test]
    fn test_eval_binary_is_not_equal() {
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(2.0), Operation::IsNotEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(1.0), Operation::IsNotEqual),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(true), Operation::IsNotEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(false), Operation::IsNotEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(true), Operation::IsNotEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(false), Operation::IsNotEqual),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::IsNotEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::IsNotEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Num(1.0), Operation::IsNotEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(false), Operation::IsNotEqual),
            RV::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Num(1.0),
                RV::Str(Arc::new("a".to_string())),
                Operation::IsNotEqual
            ),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Num(1.0),
                Operation::IsNotEqual
            ),
            RV::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("b".to_string())),
                Operation::IsNotEqual
            ),
            RV::Bool(true)
        );
    }

    #[test]
    fn test_eval_binary_less() {
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(1.0), Operation::Less),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(2.0), Operation::Less),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(2.0), RV::Num(1.0), Operation::Less),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(true), Operation::Less),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(false), Operation::Less),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(true), Operation::Less),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(false), Operation::Less),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::Less),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Less),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Num(1.0), Operation::Less),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(false), Operation::Less),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Num(1.0),
                RV::Str(Arc::new("a".to_string())),
                Operation::Less
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Num(1.0),
                Operation::Less
            ),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::Less
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("b".to_string())),
                Operation::Less
            ),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("b".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::Less
            ),
            RV::Bool(false)
        );
    }

    #[test]
    fn test_eval_binary_less_equal() {
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(1.0), Operation::LessEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(2.0), Operation::LessEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(2.0), RV::Num(1.0), Operation::LessEqual),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(true), Operation::LessEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(false), Operation::LessEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(true), Operation::LessEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(false), Operation::LessEqual),
            RV::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::LessEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::LessEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Num(1.0), Operation::LessEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(false), Operation::LessEqual),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Num(1.0),
                RV::Str(Arc::new("a".to_string())),
                Operation::LessEqual
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Num(1.0),
                Operation::LessEqual
            ),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::LessEqual
            ),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("b".to_string())),
                Operation::LessEqual
            ),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("b".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::LessEqual
            ),
            RV::Bool(false)
        );
    }

    #[test]
    fn test_eval_binary_greater() {
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(1.0), Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(2.0), Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(2.0), RV::Num(1.0), Operation::Greater),
            RV::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(true), Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(false), Operation::Greater),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(true), Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(false), Operation::Greater),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Num(1.0), Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(false), Operation::Greater),
            RV::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Num(1.0),
                RV::Str(Arc::new("a".to_string())),
                Operation::Greater
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Num(1.0),
                Operation::Greater
            ),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::Greater
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("b".to_string())),
                Operation::Greater
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("b".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::Greater
            ),
            RV::Bool(true)
        );
    }

    #[test]
    fn test_eval_binary_greater_equal() {
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(1.0), Operation::GreaterEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Num(2.0), Operation::GreaterEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(2.0), RV::Num(1.0), Operation::GreaterEqual),
            RV::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(true), Operation::GreaterEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Bool(false), Operation::GreaterEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(true), Operation::GreaterEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Bool(false), Operation::GreaterEqual),
            RV::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::GreaterEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::GreaterEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(false), RV::Num(1.0), Operation::GreaterEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(false), Operation::GreaterEqual),
            RV::Bool(true)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Num(1.0),
                RV::Str(Arc::new("a".to_string())),
                Operation::GreaterEqual
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Num(1.0),
                Operation::GreaterEqual
            ),
            RV::Bool(false)
        );
        //
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::GreaterEqual
            ),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Str(Arc::new("b".to_string())),
                Operation::GreaterEqual
            ),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("b".to_string())),
                RV::Str(Arc::new("a".to_string())),
                Operation::GreaterEqual
            ),
            RV::Bool(true)
        );
    }

    #[test]
    fn test_eval_binary_nan() {
        assert_eq!(eval_binary(RV::NaN, RV::Num(1.0), Operation::Add), RV::NaN);
        assert_eq!(eval_binary(RV::Num(1.0), RV::NaN, Operation::Add), RV::NaN);
        assert_eq!(
            eval_binary(RV::NaN, RV::Num(1.0), Operation::Subtract),
            RV::NaN
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::NaN, Operation::Subtract),
            RV::NaN
        );
        assert_eq!(
            eval_binary(RV::NaN, RV::Num(1.0), Operation::Multiply),
            RV::NaN
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::NaN, Operation::Multiply),
            RV::NaN
        );
        assert_eq!(
            eval_binary(RV::NaN, RV::Num(1.0), Operation::Divide),
            RV::NaN
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::NaN, Operation::Divide),
            RV::NaN
        );
        assert_eq!(
            eval_binary(RV::NaN, RV::Num(1.0), Operation::IsEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::NaN, Operation::IsEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::NaN, RV::Num(1.0), Operation::IsNotEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::NaN, Operation::IsNotEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::NaN, RV::Num(1.0), Operation::Less),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::NaN, Operation::Less),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::NaN, RV::Num(1.0), Operation::LessEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::NaN, Operation::LessEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::NaN, RV::Num(1.0), Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::NaN, Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::NaN, RV::Num(1.0), Operation::GreaterEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::NaN, Operation::GreaterEqual),
            RV::Bool(false)
        );
    }

    #[test]
    fn test_eval_binary_coercion() {
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Add),
            RV::Num(2.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Subtract),
            RV::Num(0.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Multiply),
            RV::Num(1.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Divide),
            RV::Num(1.0)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::IsEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::IsNotEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Less),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::LessEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Num(1.0), RV::Bool(true), Operation::GreaterEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Bool(true),
                Operation::Add
            ),
            RV::Str(Arc::new("atrue".to_string()))
        );
        assert_eq!(
            eval_binary(
                RV::Str(Arc::new("a".to_string())),
                RV::Bool(true),
                Operation::Less
            ),
            RV::Bool(false)
        );
        //

        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::Add),
            RV::Num(2.0)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::Subtract),
            RV::Num(0.0)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::Multiply),
            RV::Num(1.0)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::Divide),
            RV::Num(1.0)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::IsEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::IsNotEqual),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::Less),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::LessEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::Greater),
            RV::Bool(false)
        );
        assert_eq!(
            eval_binary(RV::Bool(true), RV::Num(1.0), Operation::GreaterEqual),
            RV::Bool(true)
        );
        assert_eq!(
            eval_binary(
                RV::Bool(true),
                RV::Str(Arc::new("a".to_string())),
                Operation::Add
            ),
            RV::Str(Arc::new("truea".to_string()))
        );
        assert_eq!(
            eval_binary(
                RV::Bool(true),
                RV::Str(Arc::new("a".to_string())),
                Operation::Less
            ),
            RV::Bool(false)
        );
    }
}
