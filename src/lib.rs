/*!
<a href="https://github.com/Nercury/little-rs">
    <img style="position: absolute; top: 0; left: 0; border: 0;" src="https://s3.amazonaws.com/github/ribbons/forkme_left_green_007200.png" alt="Fork me on GitHub">
</a>
<style>.sidebar { margin-top: 53px }</style>
*/

#![cfg_attr(feature="nightly", feature(drain, vec_resize))]

use std::collections::HashMap;
use std::io;
use std::io::Write;

mod options;
pub mod interpreter;
mod template;

pub use options::{ OptionsTemplate, Options };
pub use template::{ Template };

/// Immutable runtime parameter for machine.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Parameter(pub u32);
/// Mutable internal machine binding.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Binding(pub u32);
/// Immutable external machine function.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Call(pub u32);
/// Immutable internal machine constant.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Constant(pub u32);

/// Memory location.
///
/// This may be required by `Output`, `Push` or `Load` instructions.
#[derive(Copy, Clone)]
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
#[derive(Copy, Clone)]
pub enum Cond {
    /// Jump if stack value equals `Mem`.
    Eq,
    /// Jump if stack value not equals `Mem`.
    Ne,
    /// Jump if stack value greater than `Mem`.
    Gt,
    /// Jump if stack value less than `Mem`.
    Lt,
    /// Jump if stack value greater than or equals `Mem`.
    Gte,
    /// Jump if stack value less than or equals `Mem`.
    Lte,
}

/// Executable template instruction.
#[derive(Copy, Clone)]
pub enum Instruction {
    /// Output specified `Mem`.
    Output(Mem),
    /// Push data from `Mem` to stack.
    Push(Mem),
    /// Pop specified number of stack items.
    Pop(u16),
    /// Jump to instruction.
    Jump(u16),
    /// Jump to instruction based on `Cond`.
    CondJump(u16, Mem, Cond),
    /// Call function with specified amount of stack items and store result to stack if bool = true.
    Call(Call, u8, bool),
    /// Copy value from `Mem` to `Binding`.
    Load(Binding, Mem),
}

/// Simple value implementation.
#[derive(Clone, Eq, PartialEq, PartialOrd)]
pub enum Value {
    Null,
    Int(i64),
    Str(String)
}

impl BufferTo for Value {
    fn default() -> Value {
        Value::Null
    }

    fn buffer_to(&self, buf: &mut Vec<u8>) {
        match *self {
            Value::Null => (),
            Value::Int(ref i) => write!(buf, "{}", i).unwrap(),
            Value::Str(ref s) => write!(buf, "{}", s).unwrap(),
        }
    }
}

/// External template function.
///
/// This function is called from inside processor, and is used to implement various helpers.
pub trait Function<V> {
    fn invoke<'r>(&self, &'r [V]) -> Option<V>;
}

impl<V, F: for<'z> Fn(&'z [V]) -> Option<V>> Function<V> for F {
    fn invoke<'r>(&self, args: &'r [V]) -> Option<V> {
        self(args)
    }
}

/// Call mapping error.
#[derive(Debug)]
pub enum CallMapError {
    NotFound(String),
}

/// Converts template into a runable version.
///
/// Consumes `Template` and produces object that has `Run` trait,
/// so it is possible to call `run` on it.
///
/// Also requires `calls` list that could be mapped to calls required by processor.
pub trait BuildProcessor<'a, V> {
    type Output: Run<'a, V>;

    fn build_processor(
        &'a mut self,
        template: Template<V>,
        calls: &'a HashMap<&'a str, &'a (Function<V> + 'a)>
    ) -> Result<Self::Output, CallMapError>;
}

/// Used by processors to produce readable stream based on provided parameters.
pub trait Run<'a, V> {
    type Stream: io::Read;

    fn run(&'a self, parameters: Options<Parameter, V>) -> Self::Stream;
}

/// Writes self to growable Vec<u8> buffer.
pub trait BufferTo : Eq + PartialOrd {
    fn default() -> Self;
    fn buffer_to(&self, buf: &mut Vec<u8>);
}

/// Executes template without compilation.
pub struct Interpreter;
