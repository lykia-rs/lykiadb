use rustc_hash::FxHashMap;

use crate::value::{RV, callable::RVCallable, object::RVObject};

pub mod stdlib;

pub struct LykiaModule<'rv> {
    name: String,
    map: FxHashMap<String, RV<'rv>>,
    root: Vec<String>,
}

impl<'rv> LykiaModule<'rv> {
    pub fn new(name: &str) -> Self {
        LykiaModule {
            name: name.to_owned(),
            map: FxHashMap::default(),
            root: Vec::new(),
        }
    }

    pub fn insert(&mut self, function_name: &str, callable: RVCallable<'rv>) {
        self.map
            .insert(function_name.to_owned(), RV::Callable(callable));
    }

    pub fn insert_raw(&mut self, name: &str, value: RV<'rv>) {
        self.map.insert(name.to_owned(), value);
    }

    pub fn as_raw(&self) -> Vec<(String, RV<'rv>)> {
        let mut raw = Vec::new();
        raw.push((
            self.name.clone(),
            RV::Object(RVObject::from_map(self.map.clone())),
        ));

        for root in &self.root {
            raw.push((
                root.clone(),
                self.map.get(root).cloned().unwrap_or(RV::Undefined),
            ));
        }

        raw
    }

    pub fn expose_as_root(&mut self, name: &str) {
        self.root.push(name.to_owned());
    }
}

pub struct LykiaLibrary<'rv> {
    name: String,
    mods: Vec<LykiaModule<'rv>>,
}

impl<'rv> LykiaLibrary<'rv> {
    pub fn new(name: &str, mods: Vec<LykiaModule<'rv>>) -> Self {
        LykiaLibrary {
            name: name.to_owned(),
            mods,
        }
    }

    pub fn as_raw(&self) -> FxHashMap<String, RV<'rv>> {
        let mut lib = FxHashMap::default();
        for modl in self.mods.iter() {
            let mod_defs = modl.as_raw();
            for (name, map) in mod_defs {
                lib.insert(name, map);
            }
        }

        lib
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::{HaltReason, Interpreter};
    use crate::value::callable::Function;
    use lykiadb_common::extract;
    use lykiadb_lang::{ast::Span, types::Datatype};

    // Helper function to create a simple test callable
    fn create_test_callable(_name: &str) -> RVCallable {
        fn test_fn<'rv>(
            _: &mut Interpreter<'rv>,
            _: &Span,
            _: &[RV<'rv>],
        ) -> Result<RV<'rv>, HaltReason<'rv>> {
            Ok(RV::Undefined)
        }

        RVCallable::new(
            Function::Native { function: test_fn },
            Datatype::Unit,
            Datatype::Unit,
        )
    }

    #[test]
    fn test_lykia_module_new() {
        let module = LykiaModule::new("test_module");
        assert_eq!(module.name, "test_module");
        assert!(module.map.is_empty());
    }

    #[test]
    fn test_lykia_module_insert() {
        let mut module = LykiaModule::new("test_module");
        let callable = create_test_callable("test_fn");

        module.insert("test_function", callable);

        assert_eq!(module.map.len(), 1);
        assert!(module.map.contains_key("test_function"));
        assert!(matches!(
            module.map.get("test_function"),
            Some(RV::Callable(_))
        ));
    }

    #[test]
    fn test_lykia_module_insert_multiple() {
        let mut module = LykiaModule::new("test_module");

        module.insert("fn1", create_test_callable("fn1"));
        module.insert("fn2", create_test_callable("fn2"));
        module.insert("fn3", create_test_callable("fn3"));

        assert_eq!(module.map.len(), 3);
        assert!(module.map.contains_key("fn1"));
        assert!(module.map.contains_key("fn2"));
        assert!(module.map.contains_key("fn3"));
    }

    #[test]
    fn test_lykia_library_new() {
        let module1 = LykiaModule::new("mod1");
        let module2 = LykiaModule::new("mod2");

        let library = LykiaLibrary::new("test_lib", vec![module1, module2]);

        assert_eq!(library.name, "test_lib");
        assert_eq!(library.mods.len(), 2);
    }

    #[test]
    fn test_lykia_library_new_empty() {
        let library = LykiaLibrary::new("empty_lib", vec![]);

        assert_eq!(library.name, "empty_lib");
        assert_eq!(library.mods.len(), 0);
    }

    #[test]
    fn test_lykia_library_as_raw() {
        let mut module1 = LykiaModule::new("math");
        module1.insert("add", create_test_callable("add"));
        module1.insert("multiply", create_test_callable("multiply"));

        let mut module2 = LykiaModule::new("string");
        module2.insert("concat", create_test_callable("concat"));

        let library = LykiaLibrary::new("stdlib", vec![module1, module2]);
        let lib_map = library.as_raw();

        assert_eq!(lib_map.len(), 2);
        assert!(lib_map.contains_key("math"));
        assert!(lib_map.contains_key("string"));

        // Verify math module contents
        extract!(Some(RV::Object(math_obj)), lib_map.get("math"));

        assert_eq!(math_obj.len(), 2);
        assert!(math_obj.contains_key("add"));
        assert!(math_obj.contains_key("multiply"));

        // Verify string module contents
        extract!(Some(RV::Object(string_obj)), lib_map.get("string"));

        assert_eq!(string_obj.len(), 1);
        assert!(string_obj.contains_key("concat"));
    }

    #[test]
    fn test_lykia_library_as_raw_empty() {
        let library = LykiaLibrary::new("empty_lib", vec![]);
        let lib_map = library.as_raw();

        assert!(lib_map.is_empty());
    }

    #[test]
    fn test_lykia_module_overwrite_function() {
        let mut module = LykiaModule::new("test_module");

        module.insert("func", create_test_callable("func1"));
        assert_eq!(module.map.len(), 1);

        // Inserting with the same name should overwrite
        module.insert("func", create_test_callable("func2"));
        assert_eq!(module.map.len(), 1);
        assert!(module.map.contains_key("func"));
    }
}
