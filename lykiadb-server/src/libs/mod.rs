use rustc_hash::FxHashMap;

use crate::value::{RV, callable::RVCallable, object::RVObject};

pub mod stdlib;

pub struct LykiaModule {
    name: String,
    map: FxHashMap<String, RV>
}

impl LykiaModule {
    pub fn new(name: &str) -> Self {
        LykiaModule {
            name: name.to_owned(),
            map: FxHashMap::default()
        }
    }
    
    pub fn insert(self: &mut Self, function_name: &str, callable: RVCallable) {
        self.map.insert(function_name.to_owned(), RV::Callable(callable));
    }

    pub fn as_raw(self: &Self) -> (String, RV) {
        (self.name.clone(), RV::Object(RVObject::from_map(self.map.clone())))
    }
}

pub struct LykiaLibrary {
    name: String,
    mods: Vec<LykiaModule>
}

impl LykiaLibrary {
    pub fn new(name: &str, mods: Vec<LykiaModule>) -> Self {
        LykiaLibrary {
            name: name.to_owned(),
            mods,
        }
    }

    pub fn as_raw(self: &Self) -> FxHashMap<String, RV> {
        let mut lib = FxHashMap::default();
        for modl in self.mods.iter() {
            let (name, map) = modl.as_raw();
            lib.insert(name, map);
        }

        lib
    }
}