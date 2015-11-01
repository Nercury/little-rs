use std::collections::HashMap;
use std::io;
use {
    Run,
    Template,
    Function,
    CallMapError,
    BuildProcessor,
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

    fn run(&'a self, data: V) -> Self::Stream {
        CompilerStream
    }

    fn get_fingerprint(&self) -> [u8;20] {
        [0;20]
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
            -> Result<(), io::Error>
        where
            I: io::Read + io::Seek, O: io::Write
        {

            Ok(())
        }
    }
}
