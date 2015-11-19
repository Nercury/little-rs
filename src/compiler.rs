use std::collections::HashMap;
use std::io;
use std::fmt;
use {
    Execute,
    Fingerprint,
    Template,
    Function,
    BuildError,
    Build,
    LittleResult,
};

pub struct Compiler;

impl Compiler {
    pub fn new() -> Compiler {
        trace!("create new compiler");
        Compiler
    }
}

impl<'a, V: fmt::Debug> Build<'a, V> for Compiler {
    type Output = Executable;

    fn build(
        &'a mut self,
        id: &str,
        template: Template<V>,
        calls: &'a HashMap<&'a str, &'a (Function<V> + 'a)>
    ) -> LittleResult<Self::Output> {
        trace!("build Executable for compiler with template {:#?} and calls {:#?}", template, calls.keys().collect::<Vec<_>>());
        Ok(Executable { id: id.into() })
    }

    fn load(&'a mut self, id: &str, env: Fingerprint, calls: &'a Vec<&'a (Function<V> + 'a)>)
        -> LittleResult<Self::Output>
    {
        unreachable!("compiler load not implemented");
    }
}

pub struct Executable
{
    id: String,
}

impl<'a, V: fmt::Debug> Execute<'a, V> for Executable {
    type Stream = CompilerStream;

    fn execute(&'a self, data: V) -> Self::Stream {
        trace!("run Executable with data {:#?}", data);
        CompilerStream
    }

    fn get_id<'r>(&'r self) -> &'r str {
        &self.id
    }

    fn identify_env(&self) -> Fingerprint {
        Fingerprint::empty()
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
