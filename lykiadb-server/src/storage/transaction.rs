use lykiadb_common::memory::Shared;

use crate::storage::{Engine, engines::StorageEngine};

#[derive(Clone)]
pub struct Transaction<S: for<'a> StorageEngine<'a>> {
    store: Shared<Engine<S>>,
}