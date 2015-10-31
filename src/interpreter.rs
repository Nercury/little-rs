//! Template interpreter.

use std::io;
use std::io::{ Read, Write };
use std::collections::HashMap;
use std::borrow::Cow;

use options;

use {
    Options,
    Parameter,
    Call,
    Constant,
    Binding,
    Instruction,
    Cond,
    Mem,
    Run,
    LittleValue,
    Template,
    BuildProcessor,
    Function,
    CallMapError,
    LittleError,
};

const MAX_VALUES: usize = 500000;

/// Executes template without compilation.
pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter
    }
}

impl<'a, V: LittleValue + 'a> BuildProcessor<'a, V> for Interpreter {
    type Output = Process<'a, V>;

    /// Loads the interpreter's processor.
    ///
    /// Also maps templates call indices to runtime calls.
    fn build_processor(
        &'a mut self,
        template: Template<V>,
        calls: &'a HashMap<&'a str, &'a (Function<V> + 'a)>
    ) -> Result<Process<V>, CallMapError> {
        Ok(Process::<V> {
            instructions: template.instructions,
            constants: template.constants,
            calls:  match template.calls_template.build(calls) {
                Ok(built) => built,
                Err(options::Error::ParameterMissing(s)) => return Err(CallMapError::NotFound(s)),
            },
        })
    }
}

pub struct Process<'a, V: 'a> {
    instructions: Vec<Instruction>,
    constants: Options<Constant, V>,
    calls: Options<Call, &'a Function<V>>,
}

impl<'a, V: LittleValue + 'a> Run<'a, V> for Process<'a, V> {
    type Stream = InterpreterStream<'a, V>;

    fn run(&'a self, parameters: Options<Parameter, V>) -> InterpreterStream<'a, V> {
        InterpreterStream {
            pc: 0,
            buf: Vec::new(),
            values: Values {
                stack: Vec::new(),
                values: Vec::new(),
                process: self,
                parameters: parameters,
            }
        }
    }
}

pub struct InterpreterStream<'a, V: 'a> {
    pc: usize,
    buf: Vec<u8>,
    values: Values<'a, V>,
}

enum ExecutionResult {
    Done,
    Continue,
    Interupt,
}

impl<'a, V: LittleValue> InterpreterStream<'a, V> {
    /// Returns specified number of stack items.
    ///
    /// If stack is smaller, returns None.
    pub fn peek_stack<'r>(&'r self, slice_size: usize) -> Option<&'r [V]> {
        let stack_len = self.values.stack.len();

        if stack_len < slice_size {
            return None;
        }

        Some(&self.values.stack[stack_len - slice_size as usize .. stack_len])
    }

    fn execute(&mut self) -> Result<ExecutionResult, LittleError>  {
        match self.values.process.instructions.get(self.pc) {
            Some(i) => {
                match *i {
                    Instruction::Output(ref m) => {
                        try!(write!(self.buf, "{}", try!(self.values.get_mem_value(m))))
                    },
                    Instruction::Pop(mut c) => while c > 0 {
                        if let None = self.values.stack.pop() {
                            return Err(LittleError::StackUnderflow);
                        }
                        c -= 1;
                    },
                    Instruction::Push(ref m) => {
                        let value = try!(self.values.get_mem_value(m)).into_owned();
                        self.values.stack.push(value);
                    },
                    Instruction::Load(binding, ref m) => {
                        let value = try!(self.values.get_mem_value(m)).into_owned();
                        self.values.set(binding, value);
                    },
                    Instruction::Jump(loc) => {
                        self.pc = loc as usize;
                        return Ok(ExecutionResult::Continue);
                    },
                    Instruction::CondJump(loc, ref m, cond) => {
                        let value = try!(self.values.get_mem_value(m));
                        let value_ref = value.as_ref();
                        let stack = match self.values.stack.last() {
                            Some(value) => value,
                            None => return Err(LittleError::StackUnderflow),
                        };
                        let should_jump = match cond {
                            Cond::Eq => stack == value_ref,
                            Cond::Gt => stack > value_ref,
                            Cond::Gte => stack >= value_ref,
                            Cond::Lt => stack < value_ref,
                            Cond::Lte => stack <= value_ref,
                            Cond::Ne => stack != value_ref,
                        };
                        if should_jump {
                            self.pc = loc as usize;
                            return Ok(ExecutionResult::Continue);
                        }
                    },
                    Instruction::Call(call, argc, returns) => {
                        let fun = match self.values.process.calls.get(call) {
                            Some(f) => f,
                            None => return Err(LittleError::CallMissing(call)),
                        };

                        let stack_len = self.values.stack.len();
                        let result = fun.invoke(&self.values.stack[stack_len - argc as usize .. stack_len]);

                        if returns {
                            self.values.stack.push(result.unwrap());
                        }
                    },
                    Instruction::Interupt => {
                        self.pc += 1;
                        return Ok(ExecutionResult::Interupt);
                    }
                };
                self.pc += 1;
                Ok(ExecutionResult::Continue)
            },
            None => Ok(ExecutionResult::Done),
        }
    }

    #[cfg(feature="nightly")]
    fn consume_buf(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let self_buf_len = self.buf.len();
        if self_buf_len >= buf.len() {
            for (i, o) in self.buf.drain(..buf.len()).zip(buf.iter_mut()) {
                *o = i
            }
            Ok(buf.len())
        } else {
            for (i, o) in self.buf.drain(..).zip(&mut buf[..self_buf_len]) {
                *o = i
            }
            Ok(self_buf_len)
        }
    }

    #[cfg(not(feature="nightly"))]
    fn consume_buf(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let self_buf_len = self.buf.len();
        if self_buf_len >= buf.len() {
            for (_, o) in (0..buf.len()).zip(buf.iter_mut()) {
                *o = self.buf.remove(0);
            }
            Ok(buf.len())
        } else {
            for (_, o) in (0..self_buf_len).zip(&mut buf[..self_buf_len]) {
                *o = self.buf.remove(0)
            }
            Ok(self_buf_len)
        }
    }
}

impl<'a, V: LittleValue> io::Read for InterpreterStream<'a, V> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            if self.buf.len() >= buf.len() {
                break;
            }

            match self.execute() {
                Ok(res) => match res {
                    ExecutionResult::Done => return self.consume_buf(buf),
                    ExecutionResult::Continue => (),
                    ExecutionResult::Interupt => return Err(io::Error::new(io::ErrorKind::Other, LittleError::Interupt)),
                },
                Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidInput, e)),
            }
        }

        self.consume_buf(buf)
    }
}

struct Values<'a, V: 'a> {
    stack: Vec<V>,
    values: Vec<V>,
    parameters: Options<Parameter, V>,
    process: &'a Process<'a, V>,
}

impl<'a, V: LittleValue> Values<'a, V> {
    fn get_mem_value(&self, mem: &Mem) -> Result<Cow<V>, LittleError> {
        Ok(match *mem {
            Mem::Binding(i) => self.get(i),
            Mem::Param(i) => match self.parameters.get(i) {
                Some(value) => Cow::Borrowed(value),
                None => return Err(LittleError::ParameterMissing(i)),
            },
            Mem::Const(i) => match self.process.constants.get(i) {
                Some(value) => Cow::Borrowed(value),
                None => return Err(LittleError::ConstantMissing(i)),
            },
            Mem::StackTop1 => match self.stack.last() {
                Some(value) => Cow::Borrowed(value),
                None => return Err(LittleError::StackUnderflow),
            },
            Mem::StackTop2 => match self.stack.get(self.stack.len() - 2) {
                Some(value) => Cow::Borrowed(value),
                None => return Err(LittleError::StackUnderflow),
            },
        })
    }

    fn set(&mut self, Binding(index): Binding, value: V) {
        let i = index as usize;
        self.ensure_capacity_for_index(i);
        * unsafe { self.values.get_unchecked_mut(i) } = value;
    }

    fn get<'r>(&'r self, Binding(index): Binding) -> Cow<'r, V> {
        let i = index as usize;
        if i >= self.values.len() {
            Cow::Owned(V::default())
        } else {
            Cow::Borrowed(self.values.get(i).unwrap())
        }
    }

    #[cfg(feature="nightly")]
    fn ensure_capacity_for_index(&mut self, index: usize) {
        let required_len = index + 1;
        if required_len > MAX_VALUES {
            panic!("maximum number of values {} reached!", MAX_VALUES);
        }
        if required_len > self.values.len() {
            self.values.resize(required_len, V::default());
        }
    }

    #[cfg(not(feature="nightly"))]
    fn ensure_capacity_for_index(&mut self, index: usize) {
        use std::iter;

        let required_len = index + 1;
        if required_len > MAX_VALUES {
            panic!("maximum number of values {} reached!", MAX_VALUES);
        }
        if required_len > self.values.len() {
            let missing_len = required_len - self.values.len();
            self.values.reserve(missing_len);
            self.values.extend(iter::repeat(V::default()).take(missing_len));
        }
    }
}
