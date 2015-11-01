extern crate little;

mod mock;

use std::collections::HashMap;
use std::io::Read;
use std::error::Error;

use little::*;
use little::compiler::Compiler;

use mock::Value;

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

fn from_instructions_and_params(
    instructions: Vec<Instruction>,
    params: Vec<(Parameter, Value)>
) -> String {
    let funs = HashMap::new();
    let mut i = Compiler::new();
    let p = i.build_processor(
        Template::<Value>::empty()
            .push_instructions(instructions),
        &funs
    ).unwrap();

    let mut res = String::new();

    p.run(Options::new(params.into_iter().collect()))
        .read_to_string(&mut res)
        .unwrap();

    res
}
