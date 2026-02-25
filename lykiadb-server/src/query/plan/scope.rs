use std::collections::HashMap;

use lykiadb_lang::ast::{Identifier, sql::SqlFrom};

use super::PlannerError;

#[derive(Debug)]
pub struct Scope {
    from: HashMap<Identifier, SqlFrom>,
    // aggregates: Vec<Expr>,
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            from: HashMap::new(),
        }
    }

    pub fn add_source(&mut self, alias: &Identifier, source: SqlFrom) -> Result<(), PlannerError> {
        if self.from.contains_key(alias) {
            return Err(PlannerError::DuplicateObjectInScope(alias.clone()));
        }

        self.from.insert(alias.clone(), source);

        Ok(())
    }

    pub fn merge(&mut self, other: &Scope) -> Result<(), PlannerError> {
        for (alias, source) in &other.from {
            if self.from.contains_key(alias) {
                return Err(PlannerError::DuplicateObjectInScope(alias.clone()));
            }
            self.from.insert(alias.clone(), source.clone());
        }

        Ok(())
    }

    pub fn is_path_valid(&self, head: &Identifier, tail: &[Identifier]) -> bool {
        self.from.get(head).is_some()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// Helper function to create a test scope
    pub fn create_test_scope() -> Scope {
        Scope::new()
    }
}
