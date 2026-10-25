use std::borrow::Cow;
use bson::oid::ObjectId;

use crate::{database::error, value::RV};

pub type StorageIteratorItem<'a> = Result<(Vec<u8>, RV<'a>), error::DatabaseError>;

pub enum Key<'a> {
    Collection(Cow<'a, str>),
    Document(Cow<'a, str>, Cow<'a, ObjectId>),
}

impl<'a> Key<'a> {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Key::Collection(name) => name.as_bytes().to_vec(),
            Key::Document(coll, id) => {
                let mut k = coll.as_bytes().to_vec();
                k.push(b':');
                k.extend_from_slice(id.to_string().as_bytes());
                k
            }
        }
    }
}
