use crate::value::RV;
use lykiadb_common::testing::TestFailure;
use pretty_assertions::StrComparison;

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

    pub fn expect(&mut self, rv: String) -> Result<(), TestFailure> {
        let actual: Vec<String> = self.out.iter().map(|x| x.to_string()).collect();
        pretty_assertions::assert_eq!(actual.join("\n"), rv);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::Comparison;

    pub(crate) fn diff<L: std::fmt::Debug, R: std::fmt::Debug>(left: &L, right: &R) -> TestFailure {
        TestFailure(Comparison::new(left, right).to_string())
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
