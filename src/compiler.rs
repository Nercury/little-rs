use std::collections::HashMap;
use std::io;
use {
    Run,
    Template,
    Function,
    CallMapError,
    BuildProcessor,
    Options,
    Parameter,
};

pub struct Compiler;

impl Compiler {
    pub fn new() -> Compiler {
        Compiler
    }
}

impl<'a, V> BuildProcessor<'a, V> for Compiler {
    type Output = Process;

    fn build_processor(
        &'a mut self,
        template: Template<V>,
        calls: &'a HashMap<&'a str, &'a (Function<V> + 'a)>
    ) -> Result<Self::Output, CallMapError> {
        Ok(Process)
    }
}

pub struct Process;

impl<'a, V> Run<'a, V> for Process {
    type Stream = CompilerStream;

    fn run(&'a self, parameters: Options<Parameter, V>) -> Self::Stream {
        CompilerStream
    }
}

pub struct CompilerStream;

impl io::Read for CompilerStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Ok(0)
    }
}
