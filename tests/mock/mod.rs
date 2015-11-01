use std::fmt;

use little::{ LittleValue, LittleConstant, AsValue, FromValue };

/// Simple value implementation.
#[derive(Clone, Eq, PartialEq, PartialOrd)]
pub enum Value {
    Null,
    Int(i64),
    Str(String)
}

impl FromValue for Value {
    type Output = Value;

    fn from_value(&self) -> Option<Self::Output> {
        Some(self.clone())
    }
}

impl AsValue for Value {
    type Output = Value;

    fn as_value(&self) -> Self::Output {
        self.clone()
    }
}

impl LittleConstant for Value { }

impl LittleValue for Value {
    type Constant = Value;
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
