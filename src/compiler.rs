use std::collections::HashMap;
use std::io;
use std::fmt;
use {
    Execute,
    Fingerprint,
    Template,
    Function,
    CallMapError,
    Build,
};

pub struct Compiler;

impl Compiler {
    pub fn new() -> Compiler {
        trace!("create new compiler");
        Compiler
    }
}

impl<'a, V: fmt::Debug> Build<'a, V> for Compiler {
    type Output = Process;

    fn build(
        &'a mut self,
        template: Template<V>,
        calls: &'a HashMap<&'a str, &'a (Function<V> + 'a)>
    ) -> Result<Self::Output, CallMapError> {
        trace!("build process for compiler with template {:#?} and calls {:#?}", template, calls.keys().collect::<Vec<_>>());
        Ok(Process)
    }
}

pub struct Process;

impl<'a, V: fmt::Debug> Execute<'a, V> for Process {
    type Stream = CompilerStream;

    fn execute(&'a self, data: V) -> Self::Stream {
        trace!("run process with data {:#?}", data);
        CompilerStream
    }

    fn get_fingerprint(&self) -> Fingerprint {
        Fingerprint([0;20])
    }
}

pub struct CompilerStream;

impl io::Read for CompilerStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Ok(0)
    }
}

mod ooo {
    use std::io;

    #[allow(non_camel_case_types)]
    pub struct template_ooo;

    impl template_ooo {
        pub fn output<I, O>(input: &mut I, output: &mut O)
            -> Result<usize, io::Error>
        where
            I: io::Read + io::Seek, O: io::Write
        {

            Ok(0)
        }
    }
}
