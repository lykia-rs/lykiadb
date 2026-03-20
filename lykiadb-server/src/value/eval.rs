use super::RV;
use lykiadb_lang::ast::expr::BinaryOp;

#[inline(always)]
pub fn eval_binary<'v>(left_eval: RV<'v>, right_eval: RV<'v>, operation: BinaryOp) -> RV<'v> {
    /*
        TODO(vck):
            - Add support for object operations
            - Add support for array operations
            - Add support for function operations
    */
    match operation {
        BinaryOp::Is | BinaryOp::IsEqual => RV::Bool(left_eval == right_eval),
        BinaryOp::IsNot | BinaryOp::IsNotEqual => RV::Bool(left_eval != right_eval),
        BinaryOp::Less => RV::Bool(left_eval < right_eval),
        BinaryOp::LessEqual => RV::Bool(left_eval <= right_eval),
        BinaryOp::Greater => RV::Bool(left_eval > right_eval),
        BinaryOp::GreaterEqual => RV::Bool(left_eval >= right_eval),
        BinaryOp::Add => left_eval + right_eval,
        BinaryOp::Subtract => left_eval - right_eval,
        BinaryOp::Multiply => left_eval * right_eval,
        BinaryOp::Divide => left_eval / right_eval,
        BinaryOp::In => match left_eval.is_in(&right_eval) {
            RV::Bool(a) => RV::Bool(a),
            _ => RV::Undefined,
        },
        BinaryOp::NotIn => match left_eval.is_in(&right_eval) {
            RV::Bool(a) => RV::Bool(!a),
            _ => RV::Undefined,
        },
        // TODO: Implement operations:
        /*
           Operation::Like
           Operation::NotLike
        */
        _ => RV::Undefined,
    }
}

pub fn eval_between<'v>(subject: &RV, min: &RV, max: &RV) -> Option<bool> {
    match subject {
        RV::Double(_) => eval_between_numeric(subject, min, max),
        RV::DateTime(_) => eval_between_datetime(subject, min, max),
        RV::Str(_) => eval_between_string(subject, min, max),
        _ => None,
    }
}

fn eval_between_numeric<'v>(subject: &RV, min: &RV, max: &RV) -> Option<bool> {
    if let (Some(lower_num), Some(upper_num), Some(subject_num)) =
        (min.to_double(), max.to_double(), subject.to_double())
    {
        let min_num = lower_num.min(upper_num);
        let max_num = lower_num.max(upper_num);

        return Some(min_num <= subject_num && subject_num <= max_num);
    }

    None
}

fn eval_between_datetime(subject: &RV<'_>, min: &RV<'_>, max: &RV<'_>) -> Option<bool> {
    if let (RV::DateTime(lower_dt), RV::DateTime(upper_dt), RV::DateTime(subject_dt)) =
        (min, max, subject)
    {
        let min_dt = lower_dt.min(upper_dt);
        let max_dt = lower_dt.max(upper_dt);

        return Some(min_dt <= subject_dt && subject_dt <= max_dt);
    }

    None
}

fn eval_between_string(subject: &RV<'_>, min: &RV<'_>, max: &RV<'_>) -> Option<bool> {
    let lower_str = &min.to_string();
    let upper_str = &max.to_string();
    let subject_str = subject.to_string();

    let min_str = lower_str.min(upper_str).to_string();
    let max_str = lower_str.max(upper_str).to_string();

    Some(min_str <= subject_str && subject_str <= max_str)
}

#[cfg(test)]
mod property_tests {
    use std::sync::Arc;

    use super::*;
    use crate::value::eval::{eval_between, eval_binary};
    use lykiadb_lang::ast::expr::BinaryOp;
    use proptest::prelude::*;

    // Strategies

    fn rv_any() -> impl Strategy<Value = RV<'static>> {
        prop_oneof![
            Just(RV::Undefined),
            any::<bool>().prop_map(RV::Bool),
            any::<f64>()
                .prop_filter("finite", |x| x.is_finite())
                .prop_map(RV::Double),
            "[a-zA-Z0-9]*".prop_map(|s| RV::Str(Arc::new(s))),
        ]
    }

    fn rv_numeric() -> impl Strategy<Value = RV<'static>> {
        prop_oneof![
            any::<bool>().prop_map(RV::Bool),
            any::<f64>()
                .prop_filter("finite", |x| x.is_finite())
                .prop_map(RV::Double),
        ]
    }

    fn comparison_op() -> impl Strategy<Value = BinaryOp> {
        prop_oneof![
            Just(BinaryOp::IsEqual),
            Just(BinaryOp::IsNotEqual),
            Just(BinaryOp::Less),
            Just(BinaryOp::LessEqual),
            Just(BinaryOp::Greater),
            Just(BinaryOp::GreaterEqual),
        ]
    }

    fn all_binary_op() -> impl Strategy<Value = BinaryOp> {
        prop_oneof![
            Just(BinaryOp::Add),
            Just(BinaryOp::Subtract),
            Just(BinaryOp::Multiply),
            Just(BinaryOp::Divide),
            Just(BinaryOp::IsEqual),
            Just(BinaryOp::IsNotEqual),
            Just(BinaryOp::Less),
            Just(BinaryOp::LessEqual),
            Just(BinaryOp::Greater),
            Just(BinaryOp::GreaterEqual),
        ]
    }

    // Arithmetic: commutativity and identity

    proptest! {
        #[test]
        fn addition_is_commutative(a in rv_numeric(), b in rv_numeric()) {
            prop_assert_eq!(
                eval_binary(a.clone(), b.clone(), BinaryOp::Add),
                eval_binary(b, a, BinaryOp::Add)
            );
        }

        #[test]
        fn multiplication_is_commutative(a in rv_numeric(), b in rv_numeric()) {
            prop_assert_eq!(
                eval_binary(a.clone(), b.clone(), BinaryOp::Multiply),
                eval_binary(b, a, BinaryOp::Multiply)
            );
        }

        #[test]
        fn additive_identity(a in rv_numeric()) {
            let result = eval_binary(a.clone(), RV::Double(0.0), BinaryOp::Add);
            if let Some(n) = a.to_double() {
                prop_assert_eq!(result, RV::Double(n));
            }
        }

        #[test]
        fn multiplicative_identity(a in rv_numeric()) {
            let result = eval_binary(a.clone(), RV::Double(1.0), BinaryOp::Multiply);
            if let Some(n) = a.to_double() {
                prop_assert_eq!(result, RV::Double(n));
            }
        }

        #[test]
        fn multiply_by_zero_yields_zero(a in rv_numeric()) {
            let r1 = eval_binary(a.clone(), RV::Double(0.0), BinaryOp::Multiply);
            let r2 = eval_binary(RV::Double(0.0), a, BinaryOp::Multiply);
            prop_assert_eq!(r1, RV::Double(0.0));
            prop_assert_eq!(r2, RV::Double(0.0));
        }
    }

    // Arithmetic: division edge cases

    proptest! {
        #[test]
        fn division_by_zero(a in rv_numeric()) {
            let result = eval_binary(a.clone(), RV::Double(0.0), BinaryOp::Divide);
            if let Some(n) = a.to_double() {
                if n == 0.0 {
                    prop_assert_eq!(result, RV::Undefined);
                } else if let RV::Double(r) = result {
                    prop_assert!(r.is_infinite());
                }
            }
        }

        #[test]
        fn division_by_tiny_divisor(
            dividend in (-1e10..1e10_f64).prop_filter("finite", |x| x.is_finite()),
            divisor in (-1e-100..1e-100_f64).prop_filter("non-zero", |x| *x != 0.0)
        ) {
            if let RV::Double(n) = eval_binary(RV::Double(dividend), RV::Double(divisor), BinaryOp::Divide) {
                prop_assert!(n.is_finite() || n.is_infinite());
            }
        }

        #[test]
        fn subtraction_is_inverse_of_addition(
            a in (-1e6..1e6_f64).prop_map(RV::Double),
            b in (-1e6..1e6_f64).prop_map(RV::Double)
        ) {
            let b_abs = if let RV::Double(bv) = &b { bv.abs() } else { 0.0 };
            let sum = eval_binary(a.clone(), b.clone(), BinaryOp::Add);
            let diff = eval_binary(sum, b, BinaryOp::Subtract);
            if let (RV::Double(orig), RV::Double(result)) = (a, diff) {
                let scale = orig.abs().max(b_abs).max(1.0);
                prop_assert!((orig - result).abs() <= f64::EPSILON * scale * 100.0);
            }
        }

        #[test]
        fn tiny_number_operations_stay_finite(
            a in prop_oneof![
                (-1e-300..1e-300_f64).prop_map(RV::Double),
                any::<bool>().prop_map(RV::Bool)
            ],
            b in prop_oneof![
                (-1e-300..1e-300_f64).prop_map(RV::Double),
                any::<bool>().prop_map(RV::Bool)
            ],
            op in prop_oneof![Just(BinaryOp::Add), Just(BinaryOp::Subtract), Just(BinaryOp::Multiply)]
        ) {
            if let RV::Double(n) = eval_binary(a, b, op) {
                prop_assert!(n.is_finite() || n == 0.0);
            }
        }
    }

    // Arithmetic: NaN and infinity

    proptest! {
        #[test]
        fn nan_comparisons_always_false(
            num in any::<f64>().prop_filter("finite", |x| x.is_finite())
        ) {
            let nan = RV::Double(f64::NAN);
            let rv = RV::Double(num);
            for op in [BinaryOp::Less, BinaryOp::LessEqual, BinaryOp::Greater, BinaryOp::GreaterEqual, BinaryOp::IsEqual] {
                prop_assert_eq!(eval_binary(nan.clone(), rv.clone(), op), RV::Bool(false));
                prop_assert_eq!(eval_binary(rv.clone(), nan.clone(), op), RV::Bool(false));
            }
        }

        #[test]
        fn infinity_plus_finite_is_infinity(
            num in any::<f64>().prop_filter("finite", |x| x.is_finite())
        ) {
            let inf = RV::Double(f64::INFINITY);
            prop_assert_eq!(
                eval_binary(inf.clone(), RV::Double(num), BinaryOp::Add),
                RV::Double(f64::INFINITY)
            );
        }

        #[test]
        fn negative_infinity_plus_finite_is_negative_infinity(
            num in any::<f64>().prop_filter("finite", |x| x.is_finite())
        ) {
            let neg_inf = RV::Double(f64::NEG_INFINITY);
            prop_assert_eq!(
                eval_binary(neg_inf.clone(), RV::Double(num), BinaryOp::Add),
                RV::Double(f64::NEG_INFINITY)
            );
        }
    }

    // Comparisons: ordering properties

    proptest! {
        #[test]
        fn comparisons_return_bool(a in rv_any(), b in rv_any(), op in comparison_op()) {
            prop_assert!(matches!(eval_binary(a, b, op), RV::Bool(_)));
        }

        #[test]
        fn less_than_is_antisymmetric(a in rv_numeric(), b in rv_numeric()) {
            let a_lt_b = eval_binary(a.clone(), b.clone(), BinaryOp::Less);
            let b_lt_a = eval_binary(b, a, BinaryOp::Less);
            if a_lt_b == RV::Bool(true) {
                prop_assert_eq!(b_lt_a, RV::Bool(false));
            }
        }

        #[test]
        fn less_than_is_transitive(a in rv_numeric(), b in rv_numeric(), c in rv_numeric()) {
            let a_lt_b = eval_binary(a.clone(), b.clone(), BinaryOp::Less);
            let b_lt_c = eval_binary(b, c.clone(), BinaryOp::Less);
            let a_lt_c = eval_binary(a, c, BinaryOp::Less);
            if a_lt_b == RV::Bool(true) && b_lt_c == RV::Bool(true) {
                prop_assert_eq!(a_lt_c, RV::Bool(true));
            }
        }
    }

    // Equality properties

    proptest! {
        #[test]
        fn equality_is_reflexive(a in rv_any()) {
            prop_assert_eq!(eval_binary(a.clone(), a, BinaryOp::IsEqual), RV::Bool(true));
        }

        #[test]
        fn inequality_is_symmetric(a in rv_any(), b in rv_any()) {
            prop_assert_eq!(
                eval_binary(a.clone(), b.clone(), BinaryOp::IsNotEqual),
                eval_binary(b, a, BinaryOp::IsNotEqual)
            );
        }
    }

    // Undefined propagation

    proptest! {
        #[test]
        fn undefined_arithmetic_yields_undefined(
            a in rv_any().prop_filter("not undefined", |rv| !matches!(rv, RV::Undefined))
        ) {
            for op in [BinaryOp::Add, BinaryOp::Subtract, BinaryOp::Multiply, BinaryOp::Divide] {
                prop_assert_eq!(eval_binary(RV::Undefined, a.clone(), op), RV::Undefined);
                prop_assert_eq!(eval_binary(a.clone(), RV::Undefined, op), RV::Undefined);
            }
        }

        #[test]
        fn undefined_comparisons_yield_false(
            a in rv_any().prop_filter("not undefined", |rv| !matches!(rv, RV::Undefined))
        ) {
            for op in [BinaryOp::Less, BinaryOp::LessEqual, BinaryOp::Greater, BinaryOp::GreaterEqual] {
                prop_assert_eq!(eval_binary(RV::Undefined, a.clone(), op), RV::Bool(false));
                prop_assert_eq!(eval_binary(a.clone(), RV::Undefined, op), RV::Bool(false));
            }
        }

        #[test]
        fn undefined_equality(
            a in rv_any().prop_filter("not undefined", |rv| !matches!(rv, RV::Undefined))
        ) {
            prop_assert_eq!(eval_binary(RV::Undefined, a.clone(), BinaryOp::IsEqual), RV::Bool(false));
            prop_assert_eq!(eval_binary(a.clone(), RV::Undefined, BinaryOp::IsEqual), RV::Bool(false));
            prop_assert_eq!(eval_binary(RV::Undefined, a.clone(), BinaryOp::IsNotEqual), RV::Bool(true));
            prop_assert_eq!(eval_binary(a, RV::Undefined, BinaryOp::IsNotEqual), RV::Bool(true));
        }

        #[test]
        fn undefined_vs_undefined(op in all_binary_op()) {
            let result = eval_binary(RV::Undefined, RV::Undefined, op);
            match op {
                BinaryOp::IsEqual | BinaryOp::Is => prop_assert_eq!(result, RV::Bool(true)),
                BinaryOp::IsNotEqual | BinaryOp::IsNot => prop_assert_eq!(result, RV::Bool(false)),
                BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide => {
                    prop_assert_eq!(result, RV::Undefined);
                }
                BinaryOp::Less | BinaryOp::Greater => prop_assert_eq!(result, RV::Bool(false)),
                BinaryOp::LessEqual | BinaryOp::GreaterEqual => prop_assert_eq!(result, RV::Bool(true)),
                _ => {}
            }
        }
    }

    // Type coercion

    proptest! {
        #[test]
        fn bool_coerces_to_double_consistently(b in any::<bool>()) {
            let expected = if b { 1.0 } else { 0.0 };
            let rv = RV::Bool(b);
            prop_assert_eq!(rv.to_double(), Some(expected));
            prop_assert_eq!(eval_binary(rv, RV::Double(0.0), BinaryOp::Add), RV::Double(expected));
        }

        #[test]
        fn bool_num_mixed_arithmetic_equals_explicit_cast(
            num in any::<f64>().prop_filter("finite", |x| x.is_finite() && x.abs() < 1e100),
            b in any::<bool>(),
            op in prop_oneof![Just(BinaryOp::Add), Just(BinaryOp::Subtract), Just(BinaryOp::Multiply)]
        ) {
            let bool_as_num = RV::Double(if b { 1.0 } else { 0.0 });
            prop_assert_eq!(
                eval_binary(RV::Double(num), RV::Bool(b), op),
                eval_binary(RV::Double(num), bool_as_num, op)
            );
        }

        #[test]
        fn string_plus_bool_appends_bool_repr(s in "[a-zA-Z]{1,10}", b in any::<bool>()) {
            let bool_repr = if b { "true" } else { "false" };
            let result = eval_binary(RV::Str(Arc::new(s.clone())), RV::Bool(b), BinaryOp::Add);
            if let RV::Str(r) = result {
                let expected = format!("{s}{bool_repr}");
                prop_assert_eq!(r.as_str(), expected.as_str());
            } else {
                prop_assert!(false, "str + bool should produce a string");
            }
        }
    }

    // String operations

    proptest! {
        #[test]
        fn string_add_concatenates(s1 in "[a-zA-Z0-9]*", s2 in "[a-zA-Z0-9]*") {
            let result = eval_binary(
                RV::Str(Arc::new(s1.clone())),
                RV::Str(Arc::new(s2.clone())),
                BinaryOp::Add,
            );
            if let RV::Str(r) = result {
                let expected = format!("{s1}{s2}");
                prop_assert_eq!(r.as_str(), expected.as_str());
            } else {
                prop_assert!(false, "str + str should produce a string");
            }
        }

        #[test]
        fn string_add_unicode(
            s1 in r"[\x00-\x1F\u{1F600}-\u{1F64F}a-zA-Z0-9\s]*",
            s2 in r"[\x00-\x1F\u{1F600}-\u{1F64F}a-zA-Z0-9\s]*"
        ) {
            let result = eval_binary(
                RV::Str(Arc::new(s1.clone())),
                RV::Str(Arc::new(s2.clone())),
                BinaryOp::Add,
            );
            if let RV::Str(r) = result {
                prop_assert_eq!(r.len(), s1.len() + s2.len());
                prop_assert!(r.starts_with(&s1));
                prop_assert!(r.ends_with(&s2));
            } else {
                prop_assert!(false, "str + str should produce a string");
            }
        }

        #[test]
        fn string_non_add_is_undefined(s1 in "[a-z]{1,10}", s2 in "[a-z]{1,10}") {
            for op in [BinaryOp::Subtract, BinaryOp::Multiply, BinaryOp::Divide] {
                prop_assert_eq!(
                    eval_binary(RV::Str(Arc::new(s1.clone())), RV::Str(Arc::new(s2.clone())), op),
                    RV::Undefined
                );
            }
        }

        #[test]
        fn string_comparisons_are_lexicographic(s1 in "[a-z]{1,10}", s2 in "[a-z]{1,10}") {
            let rv1 = RV::Str(Arc::new(s1.clone()));
            let rv2 = RV::Str(Arc::new(s2.clone()));
            match s1.cmp(&s2) {
                std::cmp::Ordering::Less => {
                    prop_assert_eq!(eval_binary(rv1, rv2, BinaryOp::Less), RV::Bool(true));
                }
                std::cmp::Ordering::Equal => {
                    prop_assert_eq!(eval_binary(rv1, rv2, BinaryOp::IsEqual), RV::Bool(true));
                }
                std::cmp::Ordering::Greater => {
                    prop_assert_eq!(eval_binary(rv1, rv2, BinaryOp::Less), RV::Bool(false));
                }
            }
        }

        #[test]
        fn empty_string_is_less_than_nonempty(s in "[a-zA-Z0-9]+") {
            let empty = RV::Str(Arc::new(String::new()));
            let nonempty = RV::Str(Arc::new(s));
            prop_assert_eq!(eval_binary(empty.clone(), nonempty.clone(), BinaryOp::Less), RV::Bool(true));
            prop_assert_eq!(eval_binary(nonempty, empty, BinaryOp::Greater), RV::Bool(true));
        }

        #[test]
        fn string_comparison_always_returns_bool(
            s in r"[+-]?[0-9]+\.?[0-9]*|[a-zA-Z]+[0-9]*|[0-9]+[a-zA-Z]+"
        ) {
            let str_rv = RV::Str(Arc::new(s));
            let num_rv = RV::Double(42.0);
            prop_assert!(matches!(eval_binary(str_rv.clone(), num_rv.clone(), BinaryOp::IsEqual), RV::Bool(_)));
            prop_assert!(matches!(eval_binary(str_rv, num_rv, BinaryOp::Less), RV::Bool(_)));
        }
    }

    // eval_between: numeric

    proptest! {
        #[test]
        fn between_numeric_correctness(
            subject in any::<f64>().prop_filter("finite", |x| x.is_finite()),
            a in any::<f64>().prop_filter("finite", |x| x.is_finite()),
            b in any::<f64>().prop_filter("finite", |x| x.is_finite()),
        ) {
            let (lo, hi) = (a.min(b), a.max(b));
            let expected = lo <= subject && subject <= hi;
            prop_assert_eq!(
                eval_between(&RV::Double(subject), &RV::Double(a), &RV::Double(b)),
                Some(expected)
            );
        }

        #[test]
        fn between_numeric_bounds_are_symmetric(
            subject in any::<f64>().prop_filter("finite", |x| x.is_finite()),
            a in any::<f64>().prop_filter("finite", |x| x.is_finite()),
            b in any::<f64>().prop_filter("finite", |x| x.is_finite()),
        ) {
            prop_assert_eq!(
                eval_between(&RV::Double(subject), &RV::Double(a), &RV::Double(b)),
                eval_between(&RV::Double(subject), &RV::Double(b), &RV::Double(a))
            );
        }

        #[test]
        fn between_numeric_boundary_values_are_inclusive(
            a in any::<f64>().prop_filter("finite", |x| x.is_finite()),
            b in any::<f64>().prop_filter("finite", |x| x.is_finite()),
        ) {
            let lo = a.min(b);
            let hi = a.max(b);
            prop_assert_eq!(eval_between(&RV::Double(lo), &RV::Double(a), &RV::Double(b)), Some(true));
            prop_assert_eq!(eval_between(&RV::Double(hi), &RV::Double(a), &RV::Double(b)), Some(true));
        }
    }

    // eval_between: string

    proptest! {
        #[test]
        fn between_string_correctness(
            subject in "[a-z]{1,5}",
            a in "[a-z]{1,5}",
            b in "[a-z]{1,5}",
        ) {
            let lo = a.clone().min(b.clone());
            let hi = a.clone().max(b.clone());
            let expected = lo.as_str() <= subject.as_str() && subject.as_str() <= hi.as_str();
            prop_assert_eq!(
                eval_between(
                    &RV::Str(Arc::new(subject)),
                    &RV::Str(Arc::new(a)),
                    &RV::Str(Arc::new(b)),
                ),
                Some(expected)
            );
        }

        #[test]
        fn between_string_bounds_are_symmetric(
            subject in "[a-z]{1,5}",
            a in "[a-z]{1,5}",
            b in "[a-z]{1,5}",
        ) {
            prop_assert_eq!(
                eval_between(
                    &RV::Str(Arc::new(subject.clone())),
                    &RV::Str(Arc::new(a.clone())),
                    &RV::Str(Arc::new(b.clone())),
                ),
                eval_between(
                    &RV::Str(Arc::new(subject)),
                    &RV::Str(Arc::new(b)),
                    &RV::Str(Arc::new(a)),
                )
            );
        }

        #[test]
        fn between_string_boundary_values_are_inclusive(
            a in "[a-z]{1,5}",
            b in "[a-z]{1,5}",
        ) {
            let lo = a.clone().min(b.clone());
            let hi = a.clone().max(b.clone());
            prop_assert_eq!(
                eval_between(
                    &RV::Str(Arc::new(lo)),
                    &RV::Str(Arc::new(a.clone())),
                    &RV::Str(Arc::new(b.clone())),
                ),
                Some(true)
            );
            prop_assert_eq!(
                eval_between(
                    &RV::Str(Arc::new(hi)),
                    &RV::Str(Arc::new(a)),
                    &RV::Str(Arc::new(b)),
                ),
                Some(true)
            );
        }
    }

    // eval_between: datetime

    proptest! {
        #[test]
        fn between_datetime_correctness(
            subject_ms in any::<i64>(),
            a_ms in any::<i64>(),
            b_ms in any::<i64>(),
        ) {
            let (lo, hi) = (a_ms.min(b_ms), a_ms.max(b_ms));
            let expected = lo <= subject_ms && subject_ms <= hi;
            prop_assert_eq!(
                eval_between(
                    &RV::DateTime(bson::DateTime::from_millis(subject_ms)),
                    &RV::DateTime(bson::DateTime::from_millis(a_ms)),
                    &RV::DateTime(bson::DateTime::from_millis(b_ms)),
                ),
                Some(expected)
            );
        }

        #[test]
        fn between_datetime_bounds_are_symmetric(
            subject_ms in any::<i64>(),
            a_ms in any::<i64>(),
            b_ms in any::<i64>(),
        ) {
            let subject = bson::DateTime::from_millis(subject_ms);
            let a = bson::DateTime::from_millis(a_ms);
            let b = bson::DateTime::from_millis(b_ms);
            prop_assert_eq!(
                eval_between(&RV::DateTime(subject), &RV::DateTime(a), &RV::DateTime(b)),
                eval_between(&RV::DateTime(subject), &RV::DateTime(b), &RV::DateTime(a))
            );
        }

        #[test]
        fn between_datetime_boundary_values_are_inclusive(
            a_ms in any::<i64>(),
            b_ms in any::<i64>(),
        ) {
            let lo = bson::DateTime::from_millis(a_ms.min(b_ms));
            let hi = bson::DateTime::from_millis(a_ms.max(b_ms));
            let a = bson::DateTime::from_millis(a_ms);
            let b = bson::DateTime::from_millis(b_ms);
            prop_assert_eq!(eval_between(&RV::DateTime(lo), &RV::DateTime(a), &RV::DateTime(b)), Some(true));
            prop_assert_eq!(eval_between(&RV::DateTime(hi), &RV::DateTime(a), &RV::DateTime(b)), Some(true));
        }
    }

    // eval_between: type mismatches return None

    proptest! {
        #[test]
        fn between_unsupported_subject_returns_none(
            a in any::<f64>().prop_filter("finite", |x| x.is_finite()),
            b in any::<f64>().prop_filter("finite", |x| x.is_finite()),
            bv in any::<bool>(),
        ) {
            prop_assert_eq!(eval_between(&RV::Bool(bv), &RV::Double(a), &RV::Double(b)), None);
            prop_assert_eq!(eval_between(&RV::Undefined, &RV::Double(a), &RV::Double(b)), None);
        }

        #[test]
        fn between_mismatched_bound_types_return_none(
            subject in any::<f64>().prop_filter("finite", |x| x.is_finite()),
            a in any::<f64>().prop_filter("finite", |x| x.is_finite()),
            s in "[a-z]{1,5}",
        ) {
            prop_assert_eq!(
                eval_between(
                    &RV::Double(subject),
                    &RV::Double(a),
                    &RV::Str(Arc::new(s)),
                ),
                None
            );
        }
    }
}
