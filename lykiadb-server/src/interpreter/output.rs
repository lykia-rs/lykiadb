use crate::value::RV;

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

    pub fn expect(&mut self, rv: Vec<RV<'v>>) {
        if rv.len() == 1 {
            if let Some(first) = rv.first() {
                assert_eq!(
                    self.out.first().unwrap_or(&RV::Undefined).to_string(),
                    first.to_string()
                );
            }
        }
        assert_eq!(self.out, rv)
    }
    // TODO(vck): Remove this
    pub fn expect_str(&mut self, rv: Vec<String>) {
        assert_eq!(
            self.out
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
            rv
        )
    }
}
