#![allow(dead_code)]

use std::collections::HashMap;
use std::cmp::Ordering;
use std::fmt;

use little::{ LittleValue, IdentifyValue, Sha1Hasher, Fingerprint };

/// Simple value implementation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    Null,
    Int(i64),
    Str(String),
    Obj(HashMap<String, Value>),
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Value) -> Option<Ordering> {
        match (self, other) {
            (&Value::Null, &Value::Null) => Some(Ordering::Equal),
            (&Value::Int(ref a), &Value::Int(ref b)) => a.partial_cmp(b),
            (&Value::Str(ref a), &Value::Str(ref b)) => a.partial_cmp(b),
            (&Value::Obj(_), &Value::Obj(_)) => None,
            _ => None,
        }
    }
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
            Value::Obj(ref s) => write!(f, "{:?}", s),
        }
    }
}
