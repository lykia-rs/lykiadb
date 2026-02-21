use std::fmt::Display;

use dyn_clone::DynClone;
use interb::Symbol;
use smallvec::SmallVec;

use crate::{
    global::GLOBAL_INTERNER,
    value::{RV, object::RVObject},
};

pub type RVs<'exec> = Box<dyn RVIterator<'exec>>;

pub trait RVIterator<'exec>: Iterator<Item = ExecutionRow<'exec>> + DynClone {}

dyn_clone::clone_trait_object!(<'exec>RVIterator<'exec>);

impl<'exec, I: Iterator<Item = ExecutionRow<'exec>> + DynClone> RVIterator<'exec> for I {}
#[derive(Debug, Clone)]
pub struct ExecutionRow<'exec> {
    pub keys: SmallVec<[Symbol; 4]>,
    pub values: SmallVec<[RV<'exec>; 4]>,
}

impl<'exec> Default for ExecutionRow<'exec> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'exec> ExecutionRow<'exec> {
    pub fn new() -> Self {
        ExecutionRow {
            keys: SmallVec::new(),
            values: SmallVec::new(),
        }
    }

    pub fn get(&self, key: &Symbol) -> Option<&RV<'exec>> {
        if let Some(pos) = self.keys.iter().position(|k| k == key) {
            self.values.get(pos)
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: Symbol, value: RV<'exec>) {
        self.keys.push(key);
        self.values.push(value);
    }

    pub fn as_value(&self) -> RV<'exec> {
        let mut map = RVObject::new();
        for (k, v) in self.keys.iter().zip(self.values.iter()) {
            map.insert(GLOBAL_INTERNER.resolve(*k).unwrap().to_string(), v.clone());
        }
        RV::Object(map)
    }

    pub fn copy_to(&self, target: &mut ExecutionRow<'exec>) {
        for (k, v) in self.keys.iter().zip(self.values.iter()) {
            target.insert(*k, v.clone());
        }
    }
}

impl<'exec> Display for ExecutionRow<'exec> {
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
