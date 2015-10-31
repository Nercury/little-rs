use std::fmt;
use std::io;
use std::error;
use {
    Parameter,
    Constant,
    Call,
};

/// Runtime error.
#[derive(Debug)]
pub enum LittleError {
    /// A parameter was required for an instruction, but it was not found.
    ParameterMissing(Parameter),
    /// A constant was required for an instruction, but it was not found.
    ConstantMissing(Constant),
    /// A call was required for an instruction, but it was not found.
    CallMissing(Call),
    /// A call has returned an error.
    CallError(Box<error::Error + Sync + Send>),
    /// I/O error writing template result to output.
    OutputError(io::Error),
    /// Attempt to pop values on empty stack.
    StackUnderflow,
    /// Instruction has caused an interupt, it is up to user to know how to handle it.
    Interupt,
}

impl From<io::Error> for LittleError {
    fn from(other: io::Error) -> LittleError {
        LittleError::OutputError(other)
    }
}

impl fmt::Display for LittleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LittleError::ParameterMissing(p) => write!(f, "Parameter {:?} is missing.", p),
            LittleError::ConstantMissing(c) => write!(f, "Constant {:?} is missing.", c),
            LittleError::CallMissing(c) => write!(f, "Call {:?} is missing.", c),
            LittleError::CallError(ref e) => e.fmt(f),
            LittleError::OutputError(ref e) => write!(f, "Output error: {:?}", e),
            LittleError::StackUnderflow => write!(f, "Attempt to pop empty stack."),
            LittleError::Interupt => write!(f, "Interupt."),
        }
    }
}

impl error::Error for LittleError {
    fn description(&self) -> &str {
        match *self {
            LittleError::ParameterMissing(_) => "parameter is missing",
            LittleError::ConstantMissing(_) => "constant is missing",
            LittleError::CallMissing(_) => "call is missing",
            LittleError::CallError(ref e) => e.description(),
            LittleError::OutputError(_) => "output error",
            LittleError::StackUnderflow => "stack underflow",
            LittleError::Interupt => "interupt",
        }
    }
}

/// Runtime result.
pub type LittleResult<V> = Result<V, Box<error::Error>>;
