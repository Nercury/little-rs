//! Template interpreter.

use std::io;
use std::io::{ Read, Write };
use std::collections::HashMap;
use std::error;
use std::fmt;
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
    Interpreter,
    CallMapError,
};

pub struct InterpreterStream<'a, V: 'a> {
    pc: usize,
    buf: Vec<u8>,
    values: Values<'a, V>,
}

const MAX_VALUES: usize = 500000;

struct Values<'a, V: 'a> {
    stack: Vec<V>,
    values: Vec<V>,
    parameters: Options<Parameter, V>,
    process: &'a Process<'a, V>,
}

impl<'a, V: LittleValue> Values<'a, V> {
    fn get_mem_value(&self, mem: &Mem) -> Result<Cow<V>, Error> {
        Ok(match *mem {
            Mem::Binding(i) => self.get(i),
            Mem::Param(i) => match self.parameters.get(i) {
                Some(value) => Cow::Borrowed(value),
                None => return Err(Error::ParameterMissing(i)),
            },
            Mem::Const(i) => match self.process.constants.get(i) {
                Some(value) => Cow::Borrowed(value),
                None => return Err(Error::ConstantMissing(i)),
            },
            Mem::StackTop1 => match self.stack.last() {
                Some(value) => Cow::Borrowed(value),
                None => return Err(Error::StackUnderflow),
            },
            Mem::StackTop2 => match self.stack.get(self.stack.len() - 2) {
                Some(value) => Cow::Borrowed(value),
                None => return Err(Error::StackUnderflow),
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

#[derive(Debug)]
pub enum Error {
    ParameterMissing(Parameter),
    ConstantMissing(Constant),
    CallMissing(Call),
    CallError(Box<error::Error + Sync + Send>),
    OutputError(io::Error),
    StackUnderflow,
    Interupt,
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Error {
        Error::OutputError(other)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ParameterMissing(p) => write!(f, "Parameter {:?} is missing.", p),
            Error::ConstantMissing(c) => write!(f, "Constant {:?} is missing.", c),
            Error::CallMissing(c) => write!(f, "Call {:?} is missing.", c),
            Error::CallError(ref e) => e.fmt(f),
            Error::OutputError(ref e) => write!(f, "Output error: {:?}", e),
            Error::StackUnderflow => write!(f, "Attempt to pop empty stack."),
            Error::Interupt => write!(f, "Interupt."),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ParameterMissing(_) => "parameter is missing",
            Error::ConstantMissing(_) => "constant is missing",
            Error::CallMissing(_) => "call is missing",
            Error::CallError(ref e) => e.description(),
            Error::OutputError(_) => "output error",
            Error::StackUnderflow => "stack underflow",
            Error::Interupt => "interupt",
        }
    }
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

    fn execute(&mut self) -> Result<ExecutionResult, Error>  {
        match self.values.process.instructions.get(self.pc) {
            Some(i) => {
                match *i {
                    Instruction::Output(ref m) => {
                        try!(write!(self.buf, "{}", try!(self.values.get_mem_value(m))))
                    },
                    Instruction::Pop(mut c) => while c > 0 {
                        if let None = self.values.stack.pop() {
                            return Err(Error::StackUnderflow);
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
                            None => return Err(Error::StackUnderflow),
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
                            None => return Err(Error::CallMissing(call)),
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
                    ExecutionResult::Interupt => return Err(io::Error::new(io::ErrorKind::Other, Error::Interupt)),
                },
                Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidInput, e)),
            }
        }

        self.consume_buf(buf)
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

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::fmt;
    use std::io::Read;
    use std::io::Write;
    use std::error::Error;
    use super::super::*;

    /// Simple value implementation.
    #[derive(Clone, Eq, PartialEq, PartialOrd)]
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

    #[test]
    fn error_if_missing_param() {
        let funs = HashMap::new();
        let mut i = Interpreter::new();
        let p = i.build_processor(
            Template::empty()
                .push_instructions(vec![
                    Instruction::Output(Mem::Param(Parameter(1)))
                ]),
            &funs
        ).unwrap();

        let mut res = String::new();

        let res = p.run(Options::<Parameter, Value>::empty())
            .read_to_string(&mut res)
            .err()
            .expect("expected to receive error from read");

        assert_eq!("parameter is missing", res.description());
    }

    #[test]
    fn can_handle_interupt() {
        let funs = HashMap::new();
        let mut i = Interpreter::new();
        let p = i.build_processor(
            Template::empty()
                .push_constant(Constant(1), Value::Str("Abr".into()))
                .push_instructions(vec![
                    Instruction::Output(Mem::Const(Constant(1))),
                    Instruction::Interupt,
                    Instruction::Output(Mem::Const(Constant(1))),
                ]),
            &funs
        ).unwrap();

        let mut res = String::new();
        let mut received_interupt = false;

        let mut interpreter = p.run(Options::<Parameter, Value>::empty());
        loop {
            match interpreter.read_to_string(&mut res) {
                Err(e) => {
                    match e.description() {
                        "interupt" => received_interupt = true,
                        e => panic!("other error {:?}", e),
                    };
                },
                Ok(_) => break,
            }
        }

        assert!(received_interupt);
        assert_eq!("AbrAbr", &res);
    }

    #[test]
    fn error_if_missing_const() {
        let funs = HashMap::new();
        let mut i = Interpreter::new();
        let p = i.build_processor(
            Template::<Value>::empty()
                .push_instructions(vec![
                    Instruction::Output(Mem::Const(Constant(1)))
                ]),
            &funs
        ).unwrap();

        let mut res = String::new();

        let res = p.run(Options::empty())
            .read_to_string(&mut res)
            .err()
            .expect("expected to receive error from read");

        assert_eq!("constant is missing", res.description());
    }

    #[test]
    fn error_if_pop_empty_stack() {
        let funs = HashMap::new();
        let mut i = Interpreter::new();
        let p = i.build_processor(
            Template::empty()
                .push_instructions(vec![
                    Instruction::Pop(1)
                ]),
            &funs
        ).unwrap();

        let mut res = String::new();

        let res = p.run(Options::<Parameter, Value>::empty())
            .read_to_string(&mut res)
            .err()
            .expect("expected to receive error from read");

        assert_eq!("stack underflow", res.description());
    }

    #[test]
    fn exit() {
        let res = from_instructions_and_params(Vec::new(), Vec::new());
        assert_eq!("", res);
    }

    #[test]
    fn output_param() {
        let res = from_instructions_and_params(
            vec![
                Instruction::Output(Mem::Param(Parameter(1)))
            ],
            vec![
                (Parameter(1), Value::Str("Hello".into()))
            ]
        );

        assert_eq!("Hello", res);
    }

    #[test]
    fn should_jump() {
        let res = from_instructions_and_constants(
            vec![
                Instruction::Output(Mem::Const(Constant(1))),
                Instruction::Jump(3),
                Instruction::Output(Mem::Const(Constant(2))),
                Instruction::Output(Mem::Const(Constant(3))),
            ],
            vec![
                (Constant(1), Value::Str("Hello".into())),
                (Constant(2), Value::Str("No output".into())),
                (Constant(3), Value::Str("World".into())),
            ]
        );

        assert_eq!("HelloWorld", res);
    }

    #[test]
    fn should_jump_if_eq() {
        assert!(test_cond_jump(1, 1, Cond::Eq));
    }

    #[test]
    fn should_not_jump_if_not_eq() {
        assert!(!test_cond_jump(2, 3, Cond::Eq));
    }

    #[test]
    fn should_jump_if_gt() {
        assert!(test_cond_jump(2, 1, Cond::Gt));
    }

    #[test]
    fn should_not_jump_if_not_gt() {
        assert!(!test_cond_jump(2, 2, Cond::Gt));
        assert!(!test_cond_jump(1, 2, Cond::Gt));
    }

    #[test]
    fn should_jump_if_gte() {
        assert!(test_cond_jump(2, 1, Cond::Gte));
        assert!(test_cond_jump(2, 2, Cond::Gte));
    }

    #[test]
    fn should_not_jump_if_not_gte() {
        assert!(!test_cond_jump(1, 2, Cond::Gte));
    }

    #[test]
    fn should_jump_if_lt() {
        assert!(test_cond_jump(1, 2, Cond::Lt));
    }

    #[test]
    fn should_not_jump_if_not_lt() {
        assert!(!test_cond_jump(2, 2, Cond::Lt));
        assert!(!test_cond_jump(2, 1, Cond::Lt));
    }

    #[test]
    fn should_jump_if_lte() {
        assert!(test_cond_jump(1, 2, Cond::Lte));
        assert!(test_cond_jump(2, 2, Cond::Lte));
    }

    #[test]
    fn should_not_jump_if_not_lte() {
        assert!(!test_cond_jump(2, 1, Cond::Lte));
    }

    #[test]
    fn should_jump_if_ne() {
        assert!(test_cond_jump(2, 1, Cond::Ne));
    }

    #[test]
    fn should_not_jump_if_not_ne() {
        assert!(!test_cond_jump(2, 2, Cond::Ne));
    }

    #[test]
    fn output_const() {
        let res = from_instructions_and_constants(
            vec![
                Instruction::Output(Mem::Const(Constant(1)))
            ],
            vec![
                (Constant(1), Value::Str("Const Hello".into()))
            ]
        );

        assert_eq!("Const Hello", res);
    }

    #[test]
    fn run_function() {
        let add = |args: &[Value]| {
            Ok(match (&args[0], &args[1]) {
                (&Value::Int(a), &Value::Int(b)) => Value::Int(a + b),
                _ => unimplemented!(),
            })
        };

        let mut funs = HashMap::new();
        funs.insert("add", &add as &Function<Value>);

        let mut i = Interpreter::new();
        let p = i.build_processor(
            Template::<Value>::empty()
                .push_call("add", Call(1))
                .push_constant(Constant(1), Value::Int(2))
                .push_constant(Constant(2), Value::Int(3))
                .push_instructions(vec![
                    Instruction::Push(Mem::Const(Constant(1))),
                    Instruction::Push(Mem::Const(Constant(2))),
                    Instruction::Call(Call(1), 2, true),
                    Instruction::Output(Mem::StackTop1),
                ]),
            &funs
        ).unwrap();

        let mut res = String::new();

        p.run(Options::<Parameter, Value>::empty())
            .read_to_string(&mut res)
            .unwrap();

        assert_eq!("5", &res);
    }

    #[test]
    fn push_const_output_stack_top1() {
        let res = from_instructions_and_constants(
            vec![
                Instruction::Push(Mem::Const(Constant(1))),
                Instruction::Output(Mem::StackTop1),
            ],
            vec![
                (Constant(1), Value::Str("Hello Stack 1".into()))
            ]
        );

        assert_eq!("Hello Stack 1", res);
    }

    #[test]
    fn push_params_output_stack_top2() {
        let res = from_instructions_and_params(
            vec![
                Instruction::Push(Mem::Param(Parameter(2))),
                Instruction::Push(Mem::Param(Parameter(1))),
                Instruction::Output(Mem::StackTop2),
            ],
            vec![
                (Parameter(1), Value::Str("Do not show this".into())),
                (Parameter(2), Value::Str("Hello Stack 2".into())),
            ]
        );

        assert_eq!("Hello Stack 2", res);
    }

    #[test]
    fn load_binding_from_const_output_binding() {
        let res = from_instructions_and_constants(
            vec![
                Instruction::Load(Binding(2), Mem::Const(Constant(1))),
                Instruction::Output(Mem::Binding(Binding(2))),
            ],
            vec![
                (Constant(1), Value::Str("Hello Binding".into()))
            ]
        );

        assert_eq!("Hello Binding", res);
    }

    #[test]
    fn load_binding_from_param_output_binding() {
        let res = from_instructions_and_params(
            vec![
                Instruction::Load(Binding(0), Mem::Param(Parameter(2))),
                Instruction::Output(Mem::Binding(Binding(0))),
            ],
            vec![
                (Parameter(2), Value::Str("Hello Binding".into())),
            ]
        );

        assert_eq!("Hello Binding", res);
    }

    #[test]
    fn load_binding_from_binding_stack1_stack2_output3() {
        let res = from_instructions_and_params(
            vec![
                Instruction::Load(Binding(0), Mem::Param(Parameter(1))),
                Instruction::Load(Binding(2), Mem::Param(Parameter(2))),
                Instruction::Load(Binding(1), Mem::Binding(Binding(0))),
                Instruction::Push(Mem::Binding(Binding(2))),
                Instruction::Push(Mem::Binding(Binding(1))),
                Instruction::Load(Binding(3), Mem::StackTop1),
                Instruction::Load(Binding(4), Mem::StackTop2),
                Instruction::Output(Mem::StackTop1),
                Instruction::Output(Mem::StackTop2),
            ],
            vec![
                (Parameter(1), Value::Str("Hello".into())),
                (Parameter(2), Value::Str("World".into())),
            ]
        );

        assert_eq!("HelloWorld", res);
    }

    #[test]
    fn push_from_stack_to_stack() {
        let res = from_instructions_and_params(
            vec![
                Instruction::Push(Mem::Param(Parameter(1))),
                Instruction::Push(Mem::Param(Parameter(2))),
                Instruction::Push(Mem::StackTop1),
                Instruction::Push(Mem::StackTop2),
                Instruction::Output(Mem::StackTop1),
                Instruction::Output(Mem::StackTop2),
            ],
            vec![
                (Parameter(1), Value::Str("Hello".into())),
                (Parameter(2), Value::Str("World".into())),
            ]
        );

        assert_eq!("WorldWorld", res);
    }

    #[test]
    fn output_param_twice() {
        let res = from_instructions_and_params(
            vec![
                Instruction::Output(Mem::Param(Parameter(1))),
                Instruction::Output(Mem::Param(Parameter(1))),
            ],
            vec![
                (Parameter(1), Value::Str("Hello".into())),
            ]
        );

        assert_eq!("HelloHello", res);
    }

    #[test]
    fn output_different_params() {
        let res = from_instructions_and_params(
            vec![
                Instruction::Output(Mem::Param(Parameter(1))),
                Instruction::Output(Mem::Param(Parameter(3))),
                Instruction::Output(Mem::Param(Parameter(2))),
            ],
            vec![
                (Parameter(1), Value::Str("Hello".into())),
                (Parameter(2), Value::Str("World".into())),
                (Parameter(3), Value::Str(" ".into())),
            ]
        );

        assert_eq!("Hello World", res);
    }

    fn from_instructions_and_params(
        instructions: Vec<Instruction>,
        params: Vec<(Parameter, Value)>
    ) -> String {
        let funs = HashMap::new();
        let mut i = Interpreter::new();
        let p = i.build_processor(
            Template::empty()
                .push_instructions(instructions),
            &funs
        ).unwrap();

        let mut res = String::new();

        p.run(Options::new(params.into_iter().collect()))
            .read_to_string(&mut res)
            .unwrap();

        res
    }

    fn from_instructions_and_constants(
        instructions: Vec<Instruction>,
        constants: Vec<(Constant, Value)>
    ) -> String {
        let mut template = Template::empty()
            .push_instructions(instructions);

        for (constant, value) in constants {
            template = template.push_constant(constant, value);
        }

        let funs = HashMap::new();
        let mut i = Interpreter::new();
        let p = i.build_processor(
            template,
            &funs
        ).unwrap();

        let mut res = String::new();

        p.run(Options::empty())
            .read_to_string(&mut res)
            .unwrap();

        res
    }

    /// Check if stack compared to mem using condition produces a jump.
    fn test_cond_jump(stack: i64, mem: i64, cond: Cond) -> bool {
        let res = from_instructions_and_constants(
            vec![
                Instruction::Push(Mem::Const(Constant(2))),
                Instruction::CondJump(3, Mem::Const(Constant(1)), cond),
                Instruction::Output(Mem::Const(Constant(3))), // should continue here if not jumped
                Instruction::Output(Mem::Const(Constant(3))), // should skip to this line if jumped
            ],
            vec![
                (Constant(1), Value::Int(mem)),
                (Constant(2), Value::Int(stack)),
                (Constant(3), Value::Int(1)),
            ]
        );

        match res.as_ref() {
            "1" => true,
            "11" => false,
            v => panic!(format!("test_cond_jump produced unexpected output {:?}", v)),
        }
    }
}
