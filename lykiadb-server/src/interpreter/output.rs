use crate::value::RV;
use lykiadb_common::testing::TestFailure;
use pretty_assertions::{Comparison, StrComparison};

/// Build a `TestFailure` from a structural diff of two `Debug` values.
pub(crate) fn diff<L: std::fmt::Debug, R: std::fmt::Debug>(left: &L, right: &R) -> TestFailure {
    TestFailure(Comparison::new(left, right).to_string())
}

/// Build a `TestFailure` from a string diff of two string-like values.
pub(crate) fn str_diff(left: &str, right: &str) -> TestFailure {
    TestFailure(StrComparison::new(left, right).to_string())
}

#[derive(Clone)]
pub struct Output<'v> {
    out: Vec<RV<'v>>,
}

impl<'v> Default for Output<'v> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'v> Output<'v> {
    pub fn new() -> Output<'v> {
        Output { out: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.out.clear();
    }

    pub fn push(&mut self, rv: RV<'v>) {
        self.out.push(rv);
    }

    pub fn expect(&mut self, rv: Vec<RV<'v>>) -> Result<(), TestFailure> {
        if self.out != rv {
            return Err(diff(&self.out, &rv));
        }
        Ok(())
    }

    // TODO(vck): Remove this
    pub fn expect_str(&mut self, rv: Vec<String>) -> Result<(), TestFailure> {
        let actual: Vec<String> = self.out.iter().map(|x| x.to_string()).collect();
        if actual != rv {
            return Err(str_diff(&actual.join("\n"), &rv.join("\n")));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn push_and_expect_match() {
        let mut out = Output::new();
        out.push(RV::Double(1.0));
        assert!(out.expect(vec![RV::Double(1.0)]).is_ok());
    }

    #[test]
    fn expect_mismatch_returns_err() {
        let mut out = Output::new();
        out.push(RV::Double(1.0));
        assert!(out.expect(vec![RV::Double(2.0)]).is_err());
    }

    #[test]
    fn expect_empty_vs_nonempty() {
        let mut out = Output::new();
        assert!(out.expect(vec![RV::Double(1.0)]).is_err());
    }

    #[test]
    fn expect_str_match() {
        let mut out = Output::new();
        out.push(RV::Str(Arc::new("hello".into())));
        assert!(out.expect_str(vec!["hello".into()]).is_ok());
    }

    #[test]
    fn expect_str_mismatch_returns_err() {
        let mut out = Output::new();
        out.push(RV::Str(Arc::new("hello".into())));
        assert!(out.expect_str(vec!["world".into()]).is_err());
    }

    #[test]
    fn clear_empties_output() {
        let mut out = Output::new();
        out.push(RV::Double(42.0));
        out.clear();
        assert!(out.expect(vec![]).is_ok());
    }

    #[test]
    fn default_is_empty() {
        let mut out = Output::default();
        assert!(out.expect(vec![]).is_ok());
    }

    #[test]
    fn diff_helper_produces_failure() {
        let f = diff(&vec![1], &vec![2]);
        assert!(!f.0.is_empty());
    }

    #[test]
    fn str_diff_helper_produces_failure() {
        let f = str_diff("abc", "def");
        assert!(!f.0.is_empty());
    }
}
