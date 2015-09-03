extern crate little;

use std::collections::HashMap;
use std::io::Read;
use little::*;

fn main() {
    let funs = HashMap::new();
    let mut i = Interpreter::new();
    let p = i.build_processor(
        Template::empty()
            .push_instructions(vec![
                Instruction::Output(Mem::Const(Constant(0))),
                Instruction::Push(Mem::Param(Parameter(1))),
                Instruction::Call(Call(1), 1, true),
                Instruction::Output(Mem::StackTop1),
                Instruction::Pop(1),
            ]),
        &funs
    ).unwrap();

    let mut res = String::new();

    p.run(Options::<Parameter, Value>::empty())
        .read_to_string(&mut res)
        .unwrap();

    println!("Res: {:?}", res);
}
