use super::{RV, rv_wrapper::RvValue, bson_wrapper::BsonValue};
use bson::{Bson, Document};
use std::sync::Arc;
use rustc_hash::FxHashMap;
use crate::util::alloc_shared;

/// Conversion utilities between different ValueType implementations
/// These enable seamless interoperability between RV and BSON representations
pub struct ValueConverter;

impl ValueConverter {
    /// Convert RV to BSON
    pub fn rv_to_bson(rv: &RV) -> Bson {
        match rv {
            RV::Str(s) => Bson::String(s.as_ref().clone()),
            RV::Num(n) => Bson::Double(*n),
            RV::Bool(b) => Bson::Boolean(*b),
            RV::Array(arr) => {
                let arr_guard = arr.read().unwrap();
                let bson_array: Vec<Bson> = arr_guard
                    .iter()
                    .map(|rv| Self::rv_to_bson(rv))
                    .collect();
                Bson::Array(bson_array)
            }
            RV::Object(obj) => {
                let obj_guard = obj.read().unwrap();
                let mut doc = Document::new();
                for (key, value) in obj_guard.iter() {
                    doc.insert(key.clone(), Self::rv_to_bson(value));
                }
                Bson::Document(doc)
            }
            RV::Callable(_) => {
                // Store callable as a special document
                let mut doc = Document::new();
                doc.insert("__type", "callable");
                doc.insert("__note", "Callable conversion not fully implemented");
                Bson::Document(doc)
            }
            RV::Datatype(dt) => {
                let mut doc = Document::new();
                doc.insert("__type", "datatype");
                doc.insert("datatype", format!("{:?}", dt));
                Bson::Document(doc)
            }
            RV::Undefined => Bson::Null,
        }
    }
    
    /// Convert BSON to RV
    pub fn bson_to_rv(bson: &Bson) -> RV {
        match bson {
            Bson::String(s) => RV::Str(Arc::new(s.clone())),
            Bson::Double(n) => RV::Num(*n),
            Bson::Int32(n) => RV::Num(*n as f64),
            Bson::Int64(n) => RV::Num(*n as f64),
            Bson::Boolean(b) => RV::Bool(*b),
            Bson::Array(arr) => {
                let rv_array: Vec<RV> = arr
                    .iter()
                    .map(|bson| Self::bson_to_rv(bson))
                    .collect();
                RV::Array(alloc_shared(rv_array))
            }
            Bson::Document(doc) => {
                // Check if it's a special type
                if let Some(Bson::String(type_str)) = doc.get("__type") {
                    match type_str.as_str() {
                        "callable" => {
                            // For now, return undefined as we can't fully reconstruct callables
                            RV::Undefined
                        }
                        "datatype" => {
                            // For now, return undefined as we can't parse the datatype string
                            RV::Undefined
                        }
                        _ => {
                            // Regular object, but exclude special fields
                            let mut map = FxHashMap::default();
                            for (key, value) in doc {
                                if !key.starts_with("__") {
                                    map.insert(key.clone(), Self::bson_to_rv(value));
                                }
                            }
                            RV::Object(alloc_shared(map))
                        }
                    }
                } else {
                    // Regular object
                    let mut map = FxHashMap::default();
                    for (key, value) in doc {
                        map.insert(key.clone(), Self::bson_to_rv(value));
                    }
                    RV::Object(alloc_shared(map))
                }
            }
            Bson::Null => RV::Undefined,
            _ => RV::Undefined, // For other BSON types not supported in RV
        }
    }
    
    /// Convert RvValue to BsonValue
    pub fn rv_value_to_bson_value(rv_value: &RvValue) -> BsonValue {
        let bson = Self::rv_to_bson(rv_value.as_ref());
        BsonValue::from(bson)
    }
    
    /// Convert BsonValue to RvValue  
    pub fn bson_value_to_rv_value(bson_value: &BsonValue) -> RvValue {
        let rv = Self::bson_to_rv(bson_value.as_ref());
        RvValue::from(rv)
    }
    
    /// Batch convert Vec<RV> to Vec<Bson>
    pub fn rv_vec_to_bson_vec(rv_vec: &[RV]) -> Vec<Bson> {
        rv_vec.iter().map(|rv| Self::rv_to_bson(rv)).collect()
    }
    
    /// Batch convert Vec<Bson> to Vec<RV>
    pub fn bson_vec_to_rv_vec(bson_vec: &[Bson]) -> Vec<RV> {
        bson_vec.iter().map(|bson| Self::bson_to_rv(bson)).collect()
    }
    
    /// Convert RV map to BSON document
    pub fn rv_map_to_document(map: &FxHashMap<String, RV>) -> Document {
        let mut doc = Document::new();
        for (key, value) in map {
            doc.insert(key.clone(), Self::rv_to_bson(value));
        }
        doc
    }
    
    /// Convert BSON document to RV map
    pub fn document_to_rv_map(doc: &Document) -> FxHashMap<String, RV> {
        let mut map = FxHashMap::default();
        for (key, value) in doc {
            map.insert(key.clone(), Self::bson_to_rv(value));
        }
        map
    }
}

/// Direct conversion traits for convenience
impl From<RV> for BsonValue {
    fn from(rv: RV) -> Self {
        let bson = ValueConverter::rv_to_bson(&rv);
        BsonValue::from(bson)
    }
}

impl From<BsonValue> for RV {
    fn from(bson_value: BsonValue) -> Self {
        ValueConverter::bson_to_rv(bson_value.as_ref())
    }
}

impl From<RvValue> for BsonValue {
    fn from(rv_value: RvValue) -> Self {
        ValueConverter::rv_value_to_bson_value(&rv_value)
    }
}

impl From<BsonValue> for RvValue {
    fn from(bson_value: BsonValue) -> Self {
        ValueConverter::bson_value_to_rv_value(&bson_value)
    }
}

impl From<Bson> for RV {
    fn from(bson: Bson) -> Self {
        ValueConverter::bson_to_rv(&bson)
    }
}

impl From<RV> for Bson {
    fn from(rv: RV) -> Self {
        ValueConverter::rv_to_bson(&rv)
    }
}

/// Trait for types that can be converted between value representations
pub trait ValueConvertible<T> {
    fn convert_to(&self) -> T;
    fn convert_from(other: &T) -> Self;
}

impl ValueConvertible<BsonValue> for RvValue {
    fn convert_to(&self) -> BsonValue {
        ValueConverter::rv_value_to_bson_value(self)
    }
    
    fn convert_from(other: &BsonValue) -> Self {
        ValueConverter::bson_value_to_rv_value(other)
    }
}

impl ValueConvertible<RvValue> for BsonValue {
    fn convert_to(&self) -> RvValue {
        ValueConverter::bson_value_to_rv_value(self)
    }
    
    fn convert_from(other: &RvValue) -> Self {
        ValueConverter::rv_value_to_bson_value(other)
    }
}

/// Utility for working with mixed value types
pub struct MixedValueOperations;

impl MixedValueOperations {
    /// Compare values of different implementations (converts to common representation)
    pub fn compare_mixed(rv_value: &RvValue, bson_value: &BsonValue) -> std::cmp::Ordering {
        // Convert both to a common representation for comparison
        let rv_as_bson = ValueConverter::rv_value_to_bson_value(rv_value);
        rv_as_bson.partial_cmp(bson_value).unwrap_or(std::cmp::Ordering::Equal)
    }
    
    /// Add values of different implementations
    pub fn add_mixed(rv_value: RvValue, bson_value: BsonValue) -> BsonValue {
        let rv_as_bson = ValueConverter::rv_value_to_bson_value(&rv_value);
        rv_as_bson + bson_value
    }
    
    /// Check equality between different implementations
    pub fn eq_mixed(rv_value: &RvValue, bson_value: &BsonValue) -> bool {
        let rv_as_bson = ValueConverter::rv_value_to_bson_value(rv_value);
        rv_as_bson == *bson_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::traits::ValueType;
    
    #[test]
    fn test_rv_to_bson_conversion() {
        // Test basic types
        let rv_str = RV::Str(Arc::new("hello".to_string()));
        let bson_str = ValueConverter::rv_to_bson(&rv_str);
        assert_eq!(bson_str, Bson::String("hello".to_string()));
        
        let rv_num = RV::Num(42.0);
        let bson_num = ValueConverter::rv_to_bson(&rv_num);
        assert_eq!(bson_num, Bson::Double(42.0));
        
        let rv_bool = RV::Bool(true);
        let bson_bool = ValueConverter::rv_to_bson(&rv_bool);
        assert_eq!(bson_bool, Bson::Boolean(true));
        
        let rv_undef = RV::Undefined;
        let bson_null = ValueConverter::rv_to_bson(&rv_undef);
        assert_eq!(bson_null, Bson::Null);
    }
    
    #[test]
    fn test_bson_to_rv_conversion() {
        // Test basic types
        let bson_str = Bson::String("hello".to_string());
        let rv_str = ValueConverter::bson_to_rv(&bson_str);
        if let RV::Str(s) = rv_str {
            assert_eq!(s.as_ref(), "hello");
        } else {
            panic!("Expected RV::Str");
        }
        
        let bson_num = Bson::Double(42.0);
        let rv_num = ValueConverter::bson_to_rv(&bson_num);
        assert_eq!(rv_num, RV::Num(42.0));
        
        let bson_bool = Bson::Boolean(true);
        let rv_bool = ValueConverter::bson_to_rv(&bson_bool);
        assert_eq!(rv_bool, RV::Bool(true));
        
        let bson_null = Bson::Null;
        let rv_null = ValueConverter::bson_to_rv(&bson_null);
        assert_eq!(rv_null, RV::Undefined);
    }
    
    #[test]
    fn test_roundtrip_conversion() {
        let original_rv = RV::Num(3.14159);
        
        // RV -> BSON -> RV
        let bson = ValueConverter::rv_to_bson(&original_rv);
        let converted_rv = ValueConverter::bson_to_rv(&bson);
        
        assert_eq!(original_rv, converted_rv);
    }
    
    #[test]
    fn test_value_type_conversion() {
        let rv_value = RvValue::number(42.0);
        let bson_value = BsonValue::from(rv_value.clone());
        let converted_back = RvValue::from(bson_value);
        
        assert_eq!(rv_value.as_number(), converted_back.as_number());
        assert_eq!(rv_value.to_string(), converted_back.to_string());
    }
    
    #[test]
    fn test_mixed_operations() {
        let rv_value = RvValue::number(10.0);
        let bson_value = BsonValue::number(5.0);
        
        // Test mixed addition
        let result = MixedValueOperations::add_mixed(rv_value.clone(), bson_value.clone());
        assert_eq!(result.as_number(), Some(15.0));
        
        // Test mixed equality
        let rv_value2 = RvValue::number(5.0);
        assert!(MixedValueOperations::eq_mixed(&rv_value2, &bson_value));
        assert!(!MixedValueOperations::eq_mixed(&rv_value, &bson_value));
    }
    
    #[test]
    fn test_array_conversion() {
        let rv_array = vec![RV::Num(1.0), RV::Str(Arc::new("test".to_string())), RV::Bool(true)];
        let bson_array = ValueConverter::rv_vec_to_bson_vec(&rv_array);
        let converted_back = ValueConverter::bson_vec_to_rv_vec(&bson_array);
        
        assert_eq!(rv_array.len(), converted_back.len());
        for (original, converted) in rv_array.iter().zip(converted_back.iter()) {
            assert_eq!(original, converted);
        }
    }
    
    #[test]
    fn test_convertible_trait() {
        let rv_value = RvValue::string("hello world".to_string());
        let bson_value: BsonValue = rv_value.convert_to();
        let converted_back: RvValue = RvValue::convert_from(&bson_value);
        
        assert_eq!(rv_value.as_string(), converted_back.as_string());
    }
}