//! Template interpreter.

use std::io;
use std::io::{ Read };
use std::collections::HashMap;

use options::OptionsBuildError;

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

impl<'a, V: BufferTo> InterpreterStream<'a, V> {
    fn execute(&mut self) -> bool {
        match self.template.instructions.get(self.pc) {
            Some(i) => {
                match *i {
                    Instruction::Output(ref m) => match *m {
                        Mem::Param(i) => self.parameters[i].buffer_to(&mut self.buf),
                        // Mem::Const(i) =>
                        _ => unimplemented!(),
                    },
                    _ => unimplemented!(),
                };
                self.pc += 1;

                true
            },
            None => false,
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

            if !self.execute() {
                return self.consume_buf(buf);
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

impl<'a, V: BufferTo> Run<'a, V> for Process<'a, V> {
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

impl<'a, V: BufferTo> BuildProcessor<'a, V> for Interpreter {
    type Output = Process<'a, V>;

    /// Loads the interpreter's processor.
    ///
    /// Also maps templates function indices to runtime functions.
    fn build_processor(
        &'a mut self,
        template: Template<V>,
        functions: &'a HashMap<&'a str, &'a CallFunction>
    ) -> Result<Process<V>, FunctionMapError> {
        Ok(Process::<V> {
            instructions: template.instructions,
            constants: template.constants,
            functions: match template.functions_template.build(functions) {
                Ok(built) => built,
                Err(OptionsBuildError::ParameterMissing(s)) => return Err(FunctionMapError::NotFound(s)),
            }
        })
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::io::Read;
    use super::super::*;

    #[test]
    fn should_exit() {
        let funs = HashMap::new();
        let mut i = Interpreter::new();
        let p = i.build_processor(Template::empty(), &funs).unwrap();

        let mut res = String::new();
        p.run(Options::<Parameter, Value>::empty()).read_to_string(&mut res);

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
        ].into_iter().collect())).read_to_string(&mut res);

        assert_eq!("Hello", res);
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
        ].into_iter().collect())).read_to_string(&mut res);

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
        ].into_iter().collect())).read_to_string(&mut res);

        assert_eq!("Hello World", res);
    }
}
