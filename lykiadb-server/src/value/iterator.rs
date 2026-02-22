use std::fmt::Display;

use dyn_clone::DynClone;
use interb::Symbol;
use smallvec::SmallVec;

use crate::{
    global::GLOBAL_INTERNER,
    value::{RV, object::RVObject},
};

pub type RVs<'v> = Box<dyn RVIterator<'v> + Send>;

pub trait RVIterator<'v>: Iterator<Item = ExecutionRow<'v>> + DynClone + 'v {}

dyn_clone::clone_trait_object!(<'v> RVIterator<'v>);
impl<'v, I: Iterator<Item = ExecutionRow<'v>> + DynClone + 'v> RVIterator<'v> for I {}

#[derive(Debug, Clone)]
pub struct ExecutionRow<'v> {
    pub keys: SmallVec<[Symbol; 4]>,
    pub values: SmallVec<[RV<'v>; 4]>,
}

impl<'v> Default for ExecutionRow<'v> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'v> ExecutionRow<'v> {
    pub fn new() -> Self {
        ExecutionRow {
            keys: SmallVec::new(),
            values: SmallVec::new(),
        }
    }

    pub fn get(&self, key: &Symbol) -> Option<&RV<'v>> {
        if let Some(pos) = self.keys.iter().position(|k| k == key) {
            self.values.get(pos)
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: Symbol, value: RV<'v>) {
        self.keys.push(key);
        self.values.push(value);
    }

    pub fn as_value(&self) -> RV<'v> {
        let mut map = RVObject::new();
        for (k, v) in self.keys.iter().zip(self.values.iter()) {
            map.insert(GLOBAL_INTERNER.resolve(*k).unwrap().to_string(), v.clone());
        }
        RV::Object(map)
    }

    pub fn copy_to(&self, target: &mut ExecutionRow<'v>) {
        for (k, v) in self.keys.iter().zip(self.values.iter()) {
            target.insert(*k, v.clone());
        }
    }
}

impl<'v> Display for ExecutionRow<'v> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        write!(f, "{{")?;
        for (k, v) in self.keys.iter().zip(self.values.iter()) {
            if !first {
                write!(f, ", ")?;
            }
            first = false;
            write!(f, "{}: {}", GLOBAL_INTERNER.resolve(*k).unwrap(), v)?;
        }
        write!(f, "}}")
    }
}
