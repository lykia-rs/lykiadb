use crate::value::RV;
use lykiadb_common::testing::TestFailure;
use pretty_assertions::{Comparison, StrComparison};

/// Build a `TestFailure` from a structural diff of two `Debug` values.
pub(crate) fn diff<L: std::fmt::Debug, R: std::fmt::Debug>(left: &L, right: &R) -> TestFailure {
    TestFailure(format!("{}", Comparison::new(left, right)))
}

/// Build a `TestFailure` from a string diff of two string-like values.
pub(crate) fn str_diff(left: &str, right: &str) -> TestFailure {
    TestFailure(format!("{}", StrComparison::new(left, right)))
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
