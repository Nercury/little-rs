/*!
<a href="https://github.com/Nercury/little-rs">
    <img style="position: absolute; top: 0; left: 0; border: 0;" src="https://s3.amazonaws.com/github/ribbons/forkme_left_green_007200.png" alt="Fork me on GitHub">
</a>
<style>.sidebar { margin-top: 53px }</style>
*/

#![cfg_attr(feature="nightly", feature(test, drain))]

extern crate byteorder;
extern crate crypto;

use std::collections::HashMap;
use std::io;
use std::fmt;

mod options;
mod template;
mod error;

pub mod interpreter;
pub mod compiler;
pub mod stream;
pub mod bytecode;

pub use options::{ OptionsTemplate, Options };
pub use template::{ Template };
pub use error::seek::SeekError;
pub use error::runtime::{ LittleError, LittleResult };

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
#[derive(Copy, Clone, Debug)]
pub enum Mem {
    /// Constant item.
    Const(Constant),
    /// Binding.
    Binding(Binding),
    /// Parameter with name.
    Parameter { name: Constant },
    /// All parameters.
    Parameters,
    /// Last value on stack.
    StackTop1,
    /// Last - 1 value on stack.
    StackTop2,
}

/// Jump condition.
///
/// Used by `CondJump` instruction.
#[derive(Copy, Clone, Debug)]
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
#[derive(Copy, Clone, Debug)]
pub enum Instruction {
    /// Output specified `Mem`.
    Output { location: Mem },
    /// Put a property named `Mem` into `StackTop2` for value in `StackTop1`.
    Property { name: Mem },
    /// Push data from `Mem` to stack.
    Push { location: Mem },
    /// Pop specified number of stack items.
    Pop { times: u16 },
    /// Jump to instruction.
    Jump { pc: u16 },
    /// Jump to instruction based on `Cond`.
    CondJump { pc: u16, location: Mem, test: Cond },
    /// Call function with specified amount of stack items and store result to stack if bool = true.
    Call { call: Call, argc: u8, push_result_to_stack: bool },
    /// Copy value from `Mem` to `Binding`.
    Load { binding: Binding, location: Mem },
    /// Interupt execution, it is up to the user to know what to do with the stack at current state.
    Interupt,
}

/// External template function.
///
/// This function is called from inside processor, and is used to implement various helpers.
pub trait Function<V> {
    fn invoke<'r>(&self, &'r [V]) -> LittleResult<V>;
}

impl<V, F: for<'z> Fn(&'z [V]) -> LittleResult<V>> Function<V> for F {
    fn invoke<'r>(&self, args: &'r [V]) -> LittleResult<V> {
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

    fn run(&'a self, V) -> Self::Stream;
    fn get_fingerprint(&self) -> [u8;20];
}

pub trait LittleValue : Default + Eq + PartialOrd + Clone + fmt::Display {
    type Constant: LittleConstant;
}

/// User constant has to implement this trait.
pub trait LittleConstant : AsValue + FromValue + Eq + PartialOrd + fmt::Display { }

pub trait FromValue: Sized {
    type Output;
    fn from_value(&self) -> Option<Self::Output>;
}

pub trait AsValue {
    type Output;
    fn as_value(&self) -> Self::Output;
}

/// Seek to an offset.
pub trait PositionSeek {
    /// Seek to an offset, in position, in some container/stream.
    ///
    /// A seek beyond the end of a container is allowed, but implementation defined.
    ///
    /// If the seek operation completed successfully, this method returns the new
    /// position from the start of the container.
    fn seek(&mut self, pos: usize) -> Result<usize, SeekError>;
}
