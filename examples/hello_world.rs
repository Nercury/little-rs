extern crate little;

use std::collections::HashMap;
use std::io::{ Read, Write };
use std::fmt;

use little::*;
use little::interpreter::Interpreter;

/// Simple value implementation.
/// You can provide your own value implementation for interpreter,
/// it is generic.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub enum Value {
    Null,
    Str(String)
}

/// One requirement: this trait needs to be implemented for it.
impl LittleValue for Value { }

/// Implement hashing to fingerprint so that values can be compared using fingerprints.
/// Otherwise Little will treat all values as separate.
impl IdentifyValue for Value {
    fn identify_value(&self) -> Option<Fingerprint> {
        None
    }

    fn hash_value<H: Sha1Hasher>(&self, _hasher: &mut H) -> Result<(), ()> {
        Err(())
    }
}

/// And also requires Default trait.
impl Default for Value {
    fn default() -> Value {
        Value::Null
    }
}

/// And Display trait.
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Null => Ok(()),
            Value::Str(ref s) => write!(f, "{}", s),
        }
    }
}

/// Concatenates "Hello" and "World" and prints "Hello World"
fn main() {
    // Function that converts two values to strings and joins them.
    // This function expects to receive 2 arguments.
    // When generating instructions you should take care of that.
    let join = |args: &[Value]| {
        Ok(Value::Str(format!("{} {}", args[0], args[1])))
    };

    // Functions that can be called from template.
    let mut funs = HashMap::<&'static str, &Function<Value>>::new();
    funs.insert("join", &join);

    // Create new template with instructions and constants.
    let template = Template::empty()
        .with_instructions(vec![
            // Push constant 0 to stack. It is mapped to "Hello" in this template.
            Instruction::Push { location: Mem::Const(Constant(0)) },
            // Push template parameter 1 to stack. It will be received on the "run" call.
            Instruction::Push { location: Mem::Parameters },
            // Call function mapped to 0 with 2 arguments and put the return value in stack.
            Instruction::Call { call: Call(0), argc: 2, push_result_to_stack: true },
            // Result is on the stack, output the stack top.
            Instruction::Output { location: Mem::StackTop1 },
        ])
        // Map "join" function to 0. Actual function will be received when interpreter is
        // constructed.
        .with_call("join", Call(0))
        // Map constant "Hello" to 0.
        .with_constant(Constant(0), Value::Str("Hello".into()));

    let mut i = Interpreter::new();

    // Create the processor for this template and map its functions to function list.
    // It would fail if some functions are not found.
    let p = i.build("", template, &funs).unwrap();

    // Run template with parameters and print the output.
    let mut output = String::new();
    p.execute(
        Value::Str("World".into())
    )
        .read_to_string(&mut output)
        .unwrap();

    println!("{}", output);
}
