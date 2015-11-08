extern crate little;
#[macro_use] extern crate log;
extern crate env_logger;

mod mock;

use std::collections::HashMap;
use std::io::Read;
use std::error::Error;

use little::*;
use little::compiler::Compiler;

use mock::Value;

#[test]
fn output_param() {
    env_logger::init().unwrap();

    let funs = HashMap::new();
    let mut i = Compiler::new();
    let p = i.build(
        "",
        Template::<Value>::empty()
            .push_instructions(vec![
                Instruction::Output { location: Mem::Parameters }
            ]),
        &funs
    ).unwrap();

    let mut res = String::new();

    p.execute(Value::Str("Hello".into()))
        .read_to_string(&mut res)
        .unwrap();

    assert_eq!("Hello", res);
}
