# What are Property-Based Tests?

Property-based tests focus on testing **properties** that should hold for many different inputs, rather than testing specific input-output pairs. Instead of writing:

```rust
#[test]
fn test_addition() {
    assert_eq!(2 + 3, 5);
    assert_eq!(1 + 1, 2);
    // ... more specific cases
}
```

You write:

```rust
proptest! {
    #[test]
    fn addition_is_commutative(a: i32, b: i32) {
        prop_assert_eq!(a + b, b + a);
    }
}
```

proptest framework generates hundreds of random test cases automatically.

## Benefits of Property-Based Testing

1. **Better Coverage**: Tests many more cases than manually written unit tests
2. **Finds Edge Cases**: Often discovers bugs in corner cases you didn't think of
3. **Self-Documenting**: Properties serve as executable specifications
4. **Regression Testing**: When a property fails, it saves the minimal failing case
5. **Less Maintenance**: One property test can replace many unit tests

## Conversion Examples

### Example 1: Commutativity

**Before (Unit Test):**
```rust
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
    // ... many more specific cases
}
```

**After (Property Test):**
```rust
proptest! {
    #[test]
    fn addition_is_commutative_for_numbers(
        a in numeric_rv_strategy(),
        b in numeric_rv_strategy()
    ) {
        let result1 = eval_binary(a.clone(), b.clone(), Operation::Add);
        let result2 = eval_binary(b, a, Operation::Add);
        prop_assert_eq!(result1, result2);
    }
}
```

### Example 2: Identity Properties

**Before:**
```rust
#[test]
fn test_adding_zero() {
    assert_eq!(
        eval_binary(RV::Num(5.0), RV::Num(0.0), Operation::Add),
        RV::Num(5.0)
    );
    assert_eq!(
        eval_binary(RV::Bool(true), RV::Num(0.0), Operation::Add),
        RV::Num(1.0)
    );
    // ... more cases
}
```

**After:**
```rust
proptest! {
    #[test]
    fn adding_zero_is_identity(a in numeric_rv_strategy()) {
        let zero = RV::Num(0.0);
        let result = eval_binary(a.clone(), zero, Operation::Add);
        if let Some(num) = a.as_number() {
            prop_assert_eq!(result, RV::Num(num));
        }
    }
}
```

### Example 3: Type Properties

**Before:**
```rust
#[test]
fn test_comparisons_return_bool() {
    assert!(matches!(
        eval_binary(RV::Num(1.0), RV::Num(2.0), Operation::Less),
        RV::Bool(_)
    ));
    assert!(matches!(
        eval_binary(RV::Str(Arc::new("a".to_string())), RV::Str(Arc::new("b".to_string())), Operation::Greater),
        RV::Bool(_)
    ));
    // ... many more cases
}
```

**After:**
```rust
proptest! {
    #[test]
    fn comparisons_return_boolean(
        a in rv_strategy(),
        b in rv_strategy(),
        op in prop_oneof![
            Just(Operation::IsEqual),
            Just(Operation::IsNotEqual),
            Just(Operation::Less),
            Just(Operation::LessEqual),
            Just(Operation::Greater),
            Just(Operation::GreaterEqual)
        ]
    ) {
        let result = eval_binary(a, b, op);
        prop_assert!(matches!(result, RV::Bool(_)));
    }
}
```

## Key Components

### 1. Strategies

Strategies define how to generate test data:

```rust
// Generate any RV value
fn rv_strategy() -> impl Strategy<Value = RV> {
    prop_oneof![
        Just(RV::Undefined),
        any::<bool>().prop_map(RV::Bool),
        any::<f64>().prop_filter("finite numbers", |x| x.is_finite()).prop_map(RV::Num),
        "[a-zA-Z0-9]*".prop_map(|s| RV::Str(Arc::new(s))),
    ]
}

// Generate only numeric RV values
fn numeric_rv_strategy() -> impl Strategy<Value = RV> {
    prop_oneof![
        any::<bool>().prop_map(RV::Bool),
        any::<f64>().prop_filter("finite numbers", |x| x.is_finite()).prop_map(RV::Num),
    ]
}
```

### 2. Filters

Use filters to exclude invalid inputs:

```rust
proptest! {
    #[test]
    fn undefined_operations(
        a in rv_strategy().prop_filter("not undefined", |rv| !matches!(rv, RV::Undefined)),
        op in binary_operation_strategy()
    ) {
        // Test logic here
    }
}
```

### 3. Property Assertions

Use `prop_assert_eq!` and `prop_assert!` instead of regular assertions:

```rust
prop_assert_eq!(result1, result2);
prop_assert!(result.is_valid());
```

## Properties We Test in eval.rs

### Mathematical Properties
- **Commutativity**: `a + b = b + a`
- **Associativity**: `(a + b) + c = a + (b + c)`
- **Identity**: `a + 0 = a`, `a * 1 = a`
- **Antisymmetry**: If `a < b` then `¬(b < a)`

### Type System Properties
- **Type Preservation**: Operations return expected types
- **Coercion Consistency**: Type conversions behave consistently
- **Truthiness**: `as_bool()` behavior is consistent

### Domain-Specific Properties
- **String Concatenation**: Length relationships
- **Undefined Behavior**: Consistent handling of undefined values
- **Comparison Semantics**: Lexicographic ordering for strings

### Edge Case Properties
- **NaN Handling**: NaN comparisons always return false
- **Infinity Arithmetic**: Proper infinity handling
- **Division by Zero**: Correct edge case behavior

## When to Use Each Approach

### Use Property Tests For:
- **Mathematical operations** (arithmetic, comparisons)
- **Type system invariants**
- **Transformation properties** (parsing, serialization)
- **Algorithm properties** (sorting, searching)

### Keep Unit Tests For:
- **Specific edge cases** discovered through property testing
- **Exact output validation** for specific inputs
- **Business logic** with specific requirements
- **Integration scenarios** with known inputs/outputs

## Best Practices

1. **Start with Properties**: Think about what should always be true
2. **Use Regression Tests**: Keep unit tests for specific bugs found
3. **Combine Both**: Use property tests for general behavior, unit tests for specific cases
4. **Good Strategies**: Invest time in creating good input generators
5. **Document Properties**: Properties serve as living documentation

## Debugging Property Test Failures

When a property test fails, proptest saves the minimal failing case:

```
proptest: Saving this and future failures in /path/to/proptest-regressions/
minimal failing input: a = Undefined, op = LessEqual
```

You can then add this as a regression test:

```rust
#[test]
fn regression_undefined_less_equal() {
    // This specific case was found by property testing
    let result = eval_binary(RV::Undefined, RV::Undefined, Operation::LessEqual);
    assert_eq!(result, RV::Bool(true));
}
```

This approach gives you both the broad coverage of property-based testing and the precision of targeted unit tests for known edge cases.
