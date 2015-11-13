#![allow(dead_code)]

use std::fmt;

use little::LittleValue;

/// Simple value implementation.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub enum Value {
    Null,
    Int(i64),
    Str(String)
}

impl LittleValue for Value { }

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
