/*!
<a href="https://github.com/Nercury/little-rs">
    <img style="position: absolute; top: 0; left: 0; border: 0;" src="https://s3.amazonaws.com/github/ribbons/forkme_left_green_007200.png" alt="Fork me on GitHub">
</a>
<style>.sidebar { margin-top: 53px }</style>
*/

#![feature(drain)]

use std::collections::HashMap;
use std::io;
use std::io::Write;

mod options;
pub mod interpreter;
mod template;

pub use options::{ OptionsTemplate, Options };
pub use template::{ Template };

/// Immutable runtime parameter for machine.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Parameter(pub u32);
/// Mutable internal machine binding.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Binding(pub u32);
/// Immutable external machine function.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Function(pub u32);
/// Immutable internal machine constant.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Constant(pub u32);

/// Memory location.
///
/// This may be required by `Output`, `Push` or `Load` instructions.
pub enum Mem {
    /// Constant item.
    Const(Constant),
    /// Param item.
    Param(Parameter),
    /// Binding.
    Binding(Binding),
    /// Last value on stack.
    StackTop1,
    /// Last - 1 value on stack.
    StackTop2,
}

/// Jump condition.
///
/// Used by `CondJump` instruction.
pub enum Cond {
    /// Jump if stack value equals `Mem`.
    Eq(Mem),
    /// Jump if stack value not equals `Mem`.
    Ne(Mem),
    /// Jump if stack value greater than `Mem`.
    Gt(Mem),
    /// Jump if stack value less than `Mem`.
    Lt(Mem),
    /// Jump if stack value greater than or equals `Mem`.
    Gte(Mem),
    /// Jump if stack value less than or equals `Mem`.
    Lte(Mem),
}

/// Executable template instruction.
pub enum Instruction {
    /// Output specified `Mem`.
    Output(Mem),
    /// Push data from `Mem` to stack.
    Push(Mem),
    /// Pop specified number of stack items.
    Pop(u16),
    /// Pop stack item into specified binding.
    Store(Binding),
    /// Jump to instruction.
    Jump(u16),
    /// Jump to instruction based on `Cond`.
    CondJump(u16, Cond),
    /// Call function with specified amount of stack items and store result to stack if bool = true.
    Call(Function, u8, bool),
    /// Copy value from `Mem` to `Binding`.
    Load(Binding, Mem),
}

/// Simple value implementation.
pub enum Value {
    Null,
    Int(i64),
    Str(String)
}

impl BufferTo for Value {
    fn buffer_to(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        match *self {
            Value::Null => Ok(()),
            Value::Int(ref i) => write!(buf, "{}", i),
            Value::Str(ref s) => write!(buf, "{}", s),
        }
    }
}

/// External template function.
///
/// This function is called from inside processor, and is used to implement various helpers.
pub trait CallFunction {
    fn call_function<'a>(&'a [Value]) -> Value;
}

/// Function mapping error.
#[derive(Debug)]
pub enum FunctionMapError {
    NotFound(String),
}

/// Converts template into a runable version.
///
/// Consumes `Template` and produces object that has `Run` trait,
/// so it is possible to call `run` on it.
///
/// Also requires `functions` list that could be mapped to functions required by processor.
pub trait BuildProcessor<'a, V> {
    type Output: Run<'a, V>;

    fn build_processor(
        &'a mut self,
        template: Template<V>,
        functions: &'a HashMap<&'a str, &'a CallFunction>
    ) -> Result<Self::Output, FunctionMapError>;
}

/// Used by processors to produce readable stream based on provided parameters.
pub trait Run<'a, V> {
    type Stream: io::Read;

    fn run(&'a self, parameters: Options<Parameter, V>) -> Self::Stream;
}

/// Writes self to growable Vec<u8> buffer.
pub trait BufferTo {
    fn buffer_to(&self, buf: &mut Vec<u8>) -> io::Result<()>;
}

/// Executes template without compilation.
pub struct Interpreter;
