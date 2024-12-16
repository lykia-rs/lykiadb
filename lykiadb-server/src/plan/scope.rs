use std::collections::HashMap;

use lykiadb_lang::ast::{sql::SqlSource, Identifier};

use super::PlannerError;

#[derive(Debug)]
pub struct Scope {
    sources: HashMap<Identifier, SqlSource>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            sources: HashMap::new(),
        }
    }

    pub fn add_source(&mut self, source: SqlSource) -> Result<(), PlannerError> {
        if self.sources.contains_key(source.alias()) {
            let previous = self.sources.get(source.alias()).unwrap();
            return Err(PlannerError::DuplicateObjectInScope {
                previous: previous.alias().clone(),
                ident: source.alias().clone(),
            });
        }

        self.sources.insert(source.alias().clone(), source);

        Ok(())
    }

    pub fn merge(&mut self, other: Scope) -> Result<(), PlannerError> {
        for (_, source) in other.sources {
            self.add_source(source.clone())?;
        }

        Ok(())
    }
}
