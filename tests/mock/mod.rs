#![allow(dead_code)]

use std::fmt;

use little::{ LittleValue, IdentifyValue, Sha1Hasher, Fingerprint };

/// Simple value implementation.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub enum Value {
    Null,
    Int(i64),
    Str(String)
}

impl LittleValue for Value { }

impl IdentifyValue for Value {
    fn identify_value(&self) -> Option<Fingerprint> {
        None
    }

    fn hash_value<H: Sha1Hasher>(&self, _hasher: &mut H) -> Result<(), ()> {
        Err(())
    }
}

impl Default for Value {
    fn default() -> Value {
        Value::Null
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Null => Ok(()),
            Value::Int(ref i) => write!(f, "{}", i),
            Value::Str(ref s) => write!(f, "{}", s),
        }
    }
}
