//! Template interpreter.

use std::io;
use std::io::{ Read };
use std::collections::HashMap;
use std::error;
use std::fmt;

use options;

use {
    Options,
    Parameter,
    Function,
    Constant,
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
    template: &'a Process<'a, V>,
    parameters: Options<Parameter, V>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    ParameterMissing(Parameter),
    ConstantMissing(Constant),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ParameterMissing(p) => write!(f, "Parameter {:?} is missing.", p),
            Error::ConstantMissing(c) => write!(f, "Constant {:?} is missing.", c),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ParameterMissing(_) => "parameter is missing",
            Error::ConstantMissing(_) => "constant is missing",
        }
    }
}

impl<'a, V: BufferTo> InterpreterStream<'a, V> {
    fn execute(&mut self) -> Result<bool, Error>  {
        match self.template.instructions.get(self.pc) {
            Some(i) => {
                match *i {
                    Instruction::Output(ref m) => match *m {
                        Mem::Param(i) => match self.parameters.get(i) {
                            Some(value) => value.buffer_to(&mut self.buf),
                            None => return Err(Error::ParameterMissing(i)),
                        },
                        Mem::Const(i) => match self.template.constants.get(i) {
                            Some(value) => value.buffer_to(&mut self.buf),
                            None => return Err(Error::ConstantMissing(i)),
                        },
                        _ => unimplemented!(),
                    },
                    _ => unimplemented!(),
                };
                self.pc += 1;

                Ok(true)
            },
            None => Ok(false),
        }
    }

    fn consume_buf(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.buf.len() >= buf.len() {
            for (i, o) in self.buf.drain(..buf.len()).zip(buf.iter_mut()) {
                *o = i
            }
            Ok(buf.len())
        } else {
            let len = self.buf.len();
            for (i, o) in self.buf.drain(..).zip(&mut buf[..len]) {
                *o = i
            }
            Ok(len)
        }
    }
}

impl<'a, V: BufferTo> io::Read for InterpreterStream<'a, V> {
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
    functions: Options<Function, &'a CallFunction>,
}

impl<'a, V: BufferTo + 'a> Run<'a, V> for Process<'a, V> {
    type Stream = InterpreterStream<'a, V>;

    fn run(&'a self, parameters: Options<Parameter, V>) -> InterpreterStream<'a, V> {
        InterpreterStream {
            pc: 0,
            buf: Vec::new(),
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

impl<'a, V: BufferTo + 'a> BuildProcessor<'a, V> for Interpreter {
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
            functions:  match template.functions_template.build(functions) {
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
    use super::Error;
    use super::super::*;

    #[test]
    fn should_exit() {
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
    fn should_output_param() {
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
    fn should_produce_error_if_missing_param() {
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

        assert_eq!("parameter is missing", res.get_ref().unwrap().description());
    }

    #[test]
    fn should_output_const() {
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
    fn should_panic_if_missing_const() {
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

        assert_eq!("constant is missing", res.get_ref().unwrap().description());
    }

    #[test]
    fn should_output_param_twice() {
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
    fn should_output_different_params() {
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
