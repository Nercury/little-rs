//! Template interpreter.

use std::io;
use std::io::{ Read };
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::borrow::Cow;

use options;

use {
    Options,
    Parameter,
    Function,
    Constant,
    Binding,
    Instruction,
    Mem,
    Run,
    BufferTo,
    Template,
    BuildProcessor,
    CallFunction,
    Interpreter,
    FunctionMapError,
};

pub struct InterpreterStream<'a, V: 'a> {
    pc: usize,
    buf: Vec<u8>,
    stack: Vec<V>,
    values: Vec<V>,
    template: &'a Process<'a, V>,
    parameters: Options<Parameter, V>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    ParameterMissing(Parameter),
    ConstantMissing(Constant),
    StackUnderflow,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ParameterMissing(p) => write!(f, "Parameter {:?} is missing.", p),
            Error::ConstantMissing(c) => write!(f, "Constant {:?} is missing.", c),
            Error::StackUnderflow => write!(f, "Attempt to pop empty stack."),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ParameterMissing(_) => "parameter is missing",
            Error::ConstantMissing(_) => "constant is missing",
            Error::StackUnderflow => "stack underflow",
        }
    }
}

const MAX_VALUES: usize = 500000;

impl<'a, V: BufferTo + Clone> InterpreterStream<'a, V> {
    fn execute(&mut self) -> Result<bool, Error>  {
        match self.template.instructions.get(self.pc) {
            Some(i) => {
                match *i {
                    Instruction::Output(ref m) => match *m {
                        Mem::Binding(i) => {
                            // TODO: Move values into another object to avoid clone here.
                            let value = self.get(i).into_owned();
                            value.buffer_to(&mut self.buf);
                        },
                        Mem::Param(i) => match self.parameters.get(i) {
                            Some(value) => value.buffer_to(&mut self.buf),
                            None => return Err(Error::ParameterMissing(i)),
                        },
                        Mem::Const(i) => match self.template.constants.get(i) {
                            Some(value) => value.buffer_to(&mut self.buf),
                            None => return Err(Error::ConstantMissing(i)),
                        },
                        Mem::StackTop1 => match self.stack.last() {
                            Some(value) => value.buffer_to(&mut self.buf),
                            None => return Err(Error::StackUnderflow),
                        },
                        Mem::StackTop2 => match self.stack.get(self.stack.len() - 2) {
                            Some(value) => value.buffer_to(&mut self.buf),
                            None => return Err(Error::StackUnderflow),
                        },
                    },
                    Instruction::Pop(mut c) => while c > 0 {
                        if let None = self.stack.pop() {
                            return Err(Error::StackUnderflow);
                        }
                        c -= 1;
                    },
                    Instruction::Push(ref loc) => match *loc {
                        Mem::Binding(_i) => unimplemented!(),
                        Mem::Const(i) => self.stack.push(match self.template.constants.get(i) {
                            Some(value) => value.clone(),
                            None => return Err(Error::ConstantMissing(i)),
                        }),
                        Mem::Param(i) => self.stack.push(match self.parameters.get(i) {
                            Some(value) => value.clone(),
                            None => return Err(Error::ParameterMissing(i)),
                        }),
                        Mem::StackTop1 => unimplemented!(),
                        Mem::StackTop2 => unimplemented!(),
                    },
                    Instruction::Load(binding, ref loc) => match *loc {
                        Mem::Binding(_i) => unimplemented!(),
                        Mem::Const(i) => {
                            let value = match self.template.constants.get(i) {
                                Some(value) => value.clone(),
                                None => return Err(Error::ConstantMissing(i)),
                            };
                            self.set(binding, &value);
                        },
                        Mem::Param(i) => {
                            let value = match self.parameters.get(i) {
                                Some(value) => value.clone(),
                                None => return Err(Error::ParameterMissing(i)),
                            };
                            self.set(binding, &value);
                        },
                        Mem::StackTop1 => unimplemented!(),
                        Mem::StackTop2 => unimplemented!(),
                    },
                    _ => unimplemented!(),
                };
                self.pc += 1;

                Ok(true)
            },
            None => Ok(false),
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

    fn set(&mut self, Binding(index): Binding, value: &V) {
        let i = index as usize;
        self.ensure_capacity_for_index(i);
        * unsafe { self.values.get_unchecked_mut(i) } = value.clone();
    }

    fn get<'r>(&'r mut self, Binding(index): Binding) -> Cow<'r, V> {
        let i = index as usize;
        if i >= self.values.len() {
            Cow::Owned(V::default())
        } else {
            Cow::Borrowed(unsafe { self.values.get_unchecked(i) })
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

impl<'a, V: BufferTo + Clone> io::Read for InterpreterStream<'a, V> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            if self.buf.len() >= buf.len() {
                break;
            }

            match self.execute() {
                Ok(cont) => if !cont {
                    return self.consume_buf(buf);
                },
                Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidInput, e)),
            }
        }

        self.consume_buf(buf)
    }
}

pub struct Process<'a, V> {
    instructions: Vec<Instruction>,
    constants: Options<Constant, V>,
    _functions: Options<Function, &'a CallFunction>,
}

impl<'a, V: BufferTo + Clone + 'a> Run<'a, V> for Process<'a, V> {
    type Stream = InterpreterStream<'a, V>;

    fn run(&'a self, parameters: Options<Parameter, V>) -> InterpreterStream<'a, V> {
        InterpreterStream {
            pc: 0,
            buf: Vec::new(),
            stack: Vec::new(),
            values: Vec::new(),
            template: self,
            parameters: parameters,
        }
    }
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter
    }
}

impl<'a, V: BufferTo + Clone + 'a> BuildProcessor<'a, V> for Interpreter {
    type Output = Process<'a, V>;

    /// Loads the interpreter's processor.
    ///
    /// Also maps templates function indices to runtime functions.
    fn build_processor(
        &'a mut self,
        template: Template<V>,
        functions: &'a HashMap<&'a str, &'a (CallFunction + 'a)>
    ) -> Result<Process<V>, FunctionMapError> {
        Ok(Process::<V> {
            instructions: template.instructions,
            constants: template.constants,
            _functions:  match template.functions_template.build(functions) {
                Ok(built) => built,
                Err(options::Error::ParameterMissing(s)) => return Err(FunctionMapError::NotFound(s)),
            },
        })
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::io::Read;
    use std::error::Error;
    use super::super::*;

    #[test]
    fn exit() {
        let funs = HashMap::new();
        let mut i = Interpreter::new();
        let p = i.build_processor(Template::empty(), &funs).unwrap();

        let mut res = String::new();
        p.run(Options::<Parameter, Value>::empty())
            .read_to_string(&mut res)
            .unwrap();

        assert_eq!("", res);
    }

    #[test]
    fn output_param() {
        // higher level example
        //
        // let env = Staging::new(Interpreter::new())
        //     .push_function("do_things", || 1)
        //     .push_loader(|file| {
        //         Some(match file {
        //             "example.tmp" => Source::empty()
        //                 .push_instructions(vec![
        //                     Command::Output(Scope::Param("text"))
        //                 ])
        //                 .into()
        //                 .unwrap()
        //             _ => return None,
        //         })
        //     })
        //     .finalize()
        //     .unwrap();
        //
        // let mut res = String::new();
        //
        // let p = env.load("example.tmp")
        //     .unwrap()
        //     .run(vec![
        //         ("text", "Hello".into())
        //     ])
        //     .unwrap()
        //     .read_to_string(&mut res);

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

        p.run(Options::new(vec![
            (Parameter(1), Value::Str("Hello".into()))
        ].into_iter().collect()))
            .read_to_string(&mut res)
            .unwrap();

        assert_eq!("Hello", res);
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
    fn output_const() {
        let funs = HashMap::new();
        let mut i = Interpreter::new();
        let p = i.build_processor(
            Template::empty()
                .push_constant(Constant(1), Value::Str("Const Hello".into()))
                .push_instructions(vec![
                    Instruction::Output(Mem::Const(Constant(1)))
                ]),
            &funs
        ).unwrap();

        let mut res = String::new();

        p.run(Options::<Parameter, Value>::empty())
            .read_to_string(&mut res)
            .unwrap();

        assert_eq!("Const Hello", res);
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

        let res = p.run(Options::<Parameter, Value>::empty())
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
            Template::<Value>::empty()
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
    fn push_const_output_stack_top1() {
        let funs = HashMap::new();
        let mut i = Interpreter::new();
        let p = i.build_processor(
            Template::<Value>::empty()
                .push_constant(Constant(1), Value::Str("Hello Stack 1".into()))
                .push_instructions(vec![
                    Instruction::Push(Mem::Const(Constant(1))),
                    Instruction::Output(Mem::StackTop1),
                ]),
            &funs
        ).unwrap();

        let mut res = String::new();

        p.run(Options::empty())
            .read_to_string(&mut res)
            .unwrap();

        assert_eq!("Hello Stack 1", res);
    }

    #[test]
    fn push_params_output_stack_top2() {
        let funs = HashMap::new();
        let mut i = Interpreter::new();
        let p = i.build_processor(
            Template::<Value>::empty()
                .push_instructions(vec![
                    Instruction::Push(Mem::Param(Parameter(2))),
                    Instruction::Push(Mem::Param(Parameter(1))),
                    Instruction::Output(Mem::StackTop2),
                ]),
            &funs
        ).unwrap();

        let mut res = String::new();

        p.run(Options::new(vec![
            (Parameter(1), Value::Str("Do not show this".into())),
            (Parameter(2), Value::Str("Hello Stack 2".into())),
        ].into_iter().collect()))
            .read_to_string(&mut res)
            .unwrap();

        assert_eq!("Hello Stack 2", res);
    }

    #[test]
    fn load_binding_from_const_output_binding() {
        let funs = HashMap::new();
        let mut i = Interpreter::new();
        let p = i.build_processor(
            Template::<Value>::empty()
                .push_constant(Constant(1), Value::Str("Hello Binding".into()))
                .push_instructions(vec![
                    Instruction::Load(Binding(2), Mem::Const(Constant(1))),
                    Instruction::Output(Mem::Binding(Binding(2))),
                ]),
            &funs
        ).unwrap();

        let mut res = String::new();

        p.run(Options::empty())
            .read_to_string(&mut res)
            .unwrap();

        assert_eq!("Hello Binding", res);
    }

    #[test]
    fn load_binding_from_param_output_binding() {
        let funs = HashMap::new();
        let mut i = Interpreter::new();
        let p = i.build_processor(
            Template::<Value>::empty()
                .push_instructions(vec![
                    Instruction::Load(Binding(0), Mem::Param(Parameter(2))),
                    Instruction::Output(Mem::Binding(Binding(0))),
                ]),
            &funs
        ).unwrap();

        let mut res = String::new();

        p.run(Options::new(vec![
            (Parameter(2), Value::Str("Hello Binding".into())),
        ].into_iter().collect()))
            .read_to_string(&mut res)
            .unwrap();

        assert_eq!("Hello Binding", res);
    }

    #[test]
    fn output_param_twice() {
        let funs = HashMap::new();
        let mut i = Interpreter::new();
        let p = i.build_processor(
            Template::empty()
                .push_instructions(vec![
                    Instruction::Output(Mem::Param(Parameter(1))),
                    Instruction::Output(Mem::Param(Parameter(1))),
                ]),
            &funs
        ).unwrap();

        let mut res = String::new();

        p.run(Options::new(vec![
            (Parameter(1), Value::Str("Hello".into())),
        ].into_iter().collect()))
            .read_to_string(&mut res)
            .unwrap();

        assert_eq!("HelloHello", res);
    }

    #[test]
    fn output_different_params() {
        let funs = HashMap::new();
        let mut i = Interpreter::new();
        let p = i.build_processor(
            Template::empty()
                .push_instructions(vec![
                    Instruction::Output(Mem::Param(Parameter(1))),
                    Instruction::Output(Mem::Param(Parameter(3))),
                    Instruction::Output(Mem::Param(Parameter(2))),
                ]),
            &funs
        ).unwrap();

        let mut res = String::new();

        p.run(Options::new(vec![
            (Parameter(1), Value::Str("Hello".into())),
            (Parameter(2), Value::Str("World".into())),
            (Parameter(3), Value::Str(" ".into())),
        ].into_iter().collect()))
            .read_to_string(&mut res)
            .unwrap();

        assert_eq!("Hello World", res);
    }
}
