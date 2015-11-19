/*!
<a href="https://github.com/Nercury/little-rs">
    <img style="position: absolute; top: 0; left: 0; border: 0;" src="https://s3.amazonaws.com/github/ribbons/forkme_left_green_007200.png" alt="Fork me on GitHub">
</a>
<style>.sidebar { margin-top: 53px }</style>
*/

#![cfg_attr(feature="nightly", feature(test, drain))]

extern crate byteorder;
#[macro_use] extern crate log;

use std::collections::HashMap;
use std::io::{ self, Write };
use std::fmt;
use byteorder::{ WriteBytesExt, LittleEndian };

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
pub use error::little::{ LittleError, LittleResult };
pub use error::build::{ BuildError };

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
    /// Replace a value in `StackTop1` with its property named `Mem`.
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

/// Structure used to uniquely identify executable blobs.
#[derive(Hash, Eq, PartialEq)]
pub struct Fingerprint([u8;20]);

impl Fingerprint {
    pub fn empty() -> Fingerprint {
        Fingerprint([0;20])
    }

    pub fn new(inner: [u8;20]) -> Fingerprint {
        Fingerprint(inner)
    }
}

/// Converts template into a runable version.
///
/// Consumes `Template` and produces object that has `Run` trait,
/// so it is possible to call `run` on it.
///
/// Also requires `calls` list that could be mapped to calls required by processor.
pub trait Build<'a, V> {
    type Output: Execute<'a, V>;

    /// Builds executable from template.
    fn build(
        &'a mut self,
        id: &str,
        template: Template<V>,
        calls: &'a HashMap<&'a str, &'a (Function<V> + 'a)>
    ) -> LittleResult<Self::Output>;

    /// Loads existing executable by unique fingerprint and env fingerprint.
    fn load(&'a mut self, id: &str, env: Fingerprint, calls: &'a Vec<&'a (Function<V> + 'a)>)
        -> LittleResult<Self::Output>;
}

/// Executes compiled blob and converts input value to output stream.
pub trait Execute<'a, V> {
    type Stream: io::Read;

    /// Run this executable.
    fn execute(&'a self, V) -> Self::Stream;

    /// Get executable's id.
    fn get_id<'r>(&'r self) -> &'r str;

    /// Get environment fingerprint required by executable.
    fn identify_env(&self) -> Fingerprint;
}

pub trait IdentifyValue {
    fn identify_value(&self) -> Option<Fingerprint>;
    fn hash_value<H: Sha1Hasher>(&self, hasher: &mut H) -> Result<(), ()>;
}

pub trait Sha1Hasher {
    /// Completes a round of hashing, producing the output hash generated.
    fn finish(&self) -> Fingerprint;

    /// Writes some data into this `Hasher`
    fn write(&mut self, bytes: &[u8]);

    /// Write a single `u8` into this hasher
    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.write(&[i])
    }

    /// Write a single `u16` into this hasher.
    #[inline]
    fn write_u16(&mut self, i: u16) {
        let mut buf_ref: &mut [u8] = &mut [0u8;2];
        buf_ref.write_u16::<LittleEndian>(i).unwrap();
        self.write(buf_ref);
    }

    /// Write a single `u32` into this hasher.
    #[inline]
    fn write_u32(&mut self, i: u32) {
        let mut buf_ref: &mut [u8] = &mut [0u8;4];
        buf_ref.write_u32::<LittleEndian>(i).unwrap();
        self.write(buf_ref);
    }

    /// Write a single `u64` into this hasher.
    #[inline]
    fn write_u64(&mut self, i: u64) {
        let mut buf_ref: &mut [u8] = &mut [0u8;8];
        buf_ref.write_u64::<LittleEndian>(i).unwrap();
        self.write(buf_ref);
    }

    /// Write a single `i8` into this hasher.
    #[inline]
    fn write_i8(&mut self, i: i8) {
        let mut buf_ref: &mut [u8] = &mut [0u8;1];
        buf_ref.write_i8(i).unwrap();
        self.write(buf_ref);
    }

    /// Write a single `i16` into this hasher.
    #[inline]
    fn write_i16(&mut self, i: i16) {
        let mut buf_ref: &mut [u8] = &mut [0u8;2];
        buf_ref.write_i16::<LittleEndian>(i).unwrap();
        self.write(buf_ref);
    }

    /// Write a single `i32` into this hasher.
    #[inline]
    fn write_i32(&mut self, i: i32) {
        let mut buf_ref: &mut [u8] = &mut [0u8;4];
        buf_ref.write_i32::<LittleEndian>(i).unwrap();
        self.write(buf_ref);
    }

    /// Write a single `i64` into this hasher.
    #[inline]
    fn write_i64(&mut self, i: i64) {
        let mut buf_ref: &mut [u8] = &mut [0u8;8];
        buf_ref.write_i64::<LittleEndian>(i).unwrap();
        self.write(buf_ref);
    }
}

/// Little Value abstraction, used by runtime.
pub trait LittleValue : Default + PartialEq + PartialOrd + Clone + IdentifyValue + fmt::Display { }

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
