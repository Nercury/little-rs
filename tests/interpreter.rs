extern crate little;

mod mock;

use std::collections::HashMap;
use std::io::Read;
use std::error::Error;

use little::*;
use little::interpreter::Interpreter;

use mock::Value;

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
fn can_handle_interupt() {
    let funs = HashMap::new();
    let mut i = Interpreter::new();
    let p = i.build_processor(
        Template::empty()
            .push_constant(Constant(1), Value::Str("Abr".into()))
            .push_instructions(vec![
                Instruction::Output(Mem::Const(Constant(1))),
                Instruction::Interupt,
                Instruction::Output(Mem::Const(Constant(1))),
            ]),
        &funs
    ).unwrap();

    let mut res = String::new();
    let mut received_interupt = false;

    let mut interpreter = p.run(Options::<Parameter, Value>::empty());
    loop {
        match interpreter.read_to_string(&mut res) {
            Err(e) => {
                match e.description() {
                    "interupt" => received_interupt = true,
                    e => panic!("other error {:?}", e),
                };
            },
            Ok(_) => break,
        }
    }

    assert!(received_interupt);
    assert_eq!("AbrAbr", &res);
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

    let res = p.run(Options::empty())
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
        Template::empty()
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
fn exit() {
    let res = from_instructions_and_params(Vec::new(), Vec::new());
    assert_eq!("", res);
}

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

#[test]
fn should_jump() {
    let res = from_instructions_and_constants(
        vec![
            Instruction::Output(Mem::Const(Constant(1))),
            Instruction::Jump(3),
            Instruction::Output(Mem::Const(Constant(2))),
            Instruction::Output(Mem::Const(Constant(3))),
        ],
        vec![
            (Constant(1), Value::Str("Hello".into())),
            (Constant(2), Value::Str("No output".into())),
            (Constant(3), Value::Str("World".into())),
        ]
    );

    assert_eq!("HelloWorld", res);
}

#[test]
fn should_jump_if_eq() {
    assert!(test_cond_jump(1, 1, Cond::Eq));
}

#[test]
fn should_not_jump_if_not_eq() {
    assert!(!test_cond_jump(2, 3, Cond::Eq));
}

#[test]
fn should_jump_if_gt() {
    assert!(test_cond_jump(2, 1, Cond::Gt));
}

#[test]
fn should_not_jump_if_not_gt() {
    assert!(!test_cond_jump(2, 2, Cond::Gt));
    assert!(!test_cond_jump(1, 2, Cond::Gt));
}

#[test]
fn should_jump_if_gte() {
    assert!(test_cond_jump(2, 1, Cond::Gte));
    assert!(test_cond_jump(2, 2, Cond::Gte));
}

#[test]
fn should_not_jump_if_not_gte() {
    assert!(!test_cond_jump(1, 2, Cond::Gte));
}

#[test]
fn should_jump_if_lt() {
    assert!(test_cond_jump(1, 2, Cond::Lt));
}

#[test]
fn should_not_jump_if_not_lt() {
    assert!(!test_cond_jump(2, 2, Cond::Lt));
    assert!(!test_cond_jump(2, 1, Cond::Lt));
}

#[test]
fn should_jump_if_lte() {
    assert!(test_cond_jump(1, 2, Cond::Lte));
    assert!(test_cond_jump(2, 2, Cond::Lte));
}

#[test]
fn should_not_jump_if_not_lte() {
    assert!(!test_cond_jump(2, 1, Cond::Lte));
}

#[test]
fn should_jump_if_ne() {
    assert!(test_cond_jump(2, 1, Cond::Ne));
}

#[test]
fn should_not_jump_if_not_ne() {
    assert!(!test_cond_jump(2, 2, Cond::Ne));
}

#[test]
fn output_const() {
    let res = from_instructions_and_constants(
        vec![
            Instruction::Output(Mem::Const(Constant(1)))
        ],
        vec![
            (Constant(1), Value::Str("Const Hello".into()))
        ]
    );

    assert_eq!("Const Hello", res);
}

#[test]
fn run_function() {
    let add = |args: &[Value]| -> LittleResult<Value> {
        Ok(match (&args[0], &args[1]) {
            (&Value::Int(a), &Value::Int(b)) => Value::Int(a + b),
            _ => unimplemented!(),
        })
    };

    let mut funs = HashMap::new();
    funs.insert("add", &add as &Function<Value>);

    let mut i = Interpreter::new();
    let p = i.build_processor(
        Template::<Value>::empty()
            .push_call("add", Call(1))
            .push_constant(Constant(1), Value::Int(2))
            .push_constant(Constant(2), Value::Int(3))
            .push_instructions(vec![
                Instruction::Push(Mem::Const(Constant(1))),
                Instruction::Push(Mem::Const(Constant(2))),
                Instruction::Call(Call(1), 2, true),
                Instruction::Output(Mem::StackTop1),
            ]),
        &funs
    ).unwrap();

    let mut res = String::new();

    p.run(Options::<Parameter, Value>::empty())
        .read_to_string(&mut res)
        .unwrap();

    assert_eq!("5", &res);
}

#[test]
fn push_const_output_stack_top1() {
    let res = from_instructions_and_constants(
        vec![
            Instruction::Push(Mem::Const(Constant(1))),
            Instruction::Output(Mem::StackTop1),
        ],
        vec![
            (Constant(1), Value::Str("Hello Stack 1".into()))
        ]
    );

    assert_eq!("Hello Stack 1", res);
}

#[test]
fn push_params_output_stack_top2() {
    let res = from_instructions_and_params(
        vec![
            Instruction::Push(Mem::Param(Parameter(2))),
            Instruction::Push(Mem::Param(Parameter(1))),
            Instruction::Output(Mem::StackTop2),
        ],
        vec![
            (Parameter(1), Value::Str("Do not show this".into())),
            (Parameter(2), Value::Str("Hello Stack 2".into())),
        ]
    );

    assert_eq!("Hello Stack 2", res);
}

#[test]
fn load_binding_from_const_output_binding() {
    let res = from_instructions_and_constants(
        vec![
            Instruction::Load(Binding(2), Mem::Const(Constant(1))),
            Instruction::Output(Mem::Binding(Binding(2))),
        ],
        vec![
            (Constant(1), Value::Str("Hello Binding".into()))
        ]
    );

    assert_eq!("Hello Binding", res);
}

#[test]
fn load_binding_from_param_output_binding() {
    let res = from_instructions_and_params(
        vec![
            Instruction::Load(Binding(0), Mem::Param(Parameter(2))),
            Instruction::Output(Mem::Binding(Binding(0))),
        ],
        vec![
            (Parameter(2), Value::Str("Hello Binding".into())),
        ]
    );

    assert_eq!("Hello Binding", res);
}

#[test]
fn load_binding_from_binding_stack1_stack2_output3() {
    let res = from_instructions_and_params(
        vec![
            Instruction::Load(Binding(0), Mem::Param(Parameter(1))),
            Instruction::Load(Binding(2), Mem::Param(Parameter(2))),
            Instruction::Load(Binding(1), Mem::Binding(Binding(0))),
            Instruction::Push(Mem::Binding(Binding(2))),
            Instruction::Push(Mem::Binding(Binding(1))),
            Instruction::Load(Binding(3), Mem::StackTop1),
            Instruction::Load(Binding(4), Mem::StackTop2),
            Instruction::Output(Mem::StackTop1),
            Instruction::Output(Mem::StackTop2),
        ],
        vec![
            (Parameter(1), Value::Str("Hello".into())),
            (Parameter(2), Value::Str("World".into())),
        ]
    );

    assert_eq!("HelloWorld", res);
}

#[test]
fn push_from_stack_to_stack() {
    let res = from_instructions_and_params(
        vec![
            Instruction::Push(Mem::Param(Parameter(1))),
            Instruction::Push(Mem::Param(Parameter(2))),
            Instruction::Push(Mem::StackTop1),
            Instruction::Push(Mem::StackTop2),
            Instruction::Output(Mem::StackTop1),
            Instruction::Output(Mem::StackTop2),
        ],
        vec![
            (Parameter(1), Value::Str("Hello".into())),
            (Parameter(2), Value::Str("World".into())),
        ]
    );

    assert_eq!("WorldWorld", res);
}

#[test]
fn output_param_twice() {
    let res = from_instructions_and_params(
        vec![
            Instruction::Output(Mem::Param(Parameter(1))),
            Instruction::Output(Mem::Param(Parameter(1))),
        ],
        vec![
            (Parameter(1), Value::Str("Hello".into())),
        ]
    );

    assert_eq!("HelloHello", res);
}

#[test]
fn output_different_params() {
    let res = from_instructions_and_params(
        vec![
            Instruction::Output(Mem::Param(Parameter(1))),
            Instruction::Output(Mem::Param(Parameter(3))),
            Instruction::Output(Mem::Param(Parameter(2))),
        ],
        vec![
            (Parameter(1), Value::Str("Hello".into())),
            (Parameter(2), Value::Str("World".into())),
            (Parameter(3), Value::Str(" ".into())),
        ]
    );

    assert_eq!("Hello World", res);
}

fn from_instructions_and_params(
    instructions: Vec<Instruction>,
    params: Vec<(Parameter, Value)>
) -> String {
    let funs = HashMap::new();
    let mut i = Interpreter::new();
    let p = i.build_processor(
        Template::empty()
            .push_instructions(instructions),
        &funs
    ).unwrap();

    let mut res = String::new();

    p.run(Options::new(params.into_iter().collect()))
        .read_to_string(&mut res)
        .unwrap();

    res
}

fn from_instructions_and_constants(
    instructions: Vec<Instruction>,
    constants: Vec<(Constant, Value)>
) -> String {
    let mut template = Template::empty()
        .push_instructions(instructions);

    for (constant, value) in constants {
        template = template.push_constant(constant, value);
    }

    let funs = HashMap::new();
    let mut i = Interpreter::new();
    let p = i.build_processor(
        template,
        &funs
    ).unwrap();

    let mut res = String::new();

    p.run(Options::empty())
        .read_to_string(&mut res)
        .unwrap();

    res
}

/// Check if stack compared to mem using condition produces a jump.
fn test_cond_jump(stack: i64, mem: i64, cond: Cond) -> bool {
    let res = from_instructions_and_constants(
        vec![
            Instruction::Push(Mem::Const(Constant(2))),
            Instruction::CondJump(3, Mem::Const(Constant(1)), cond),
            Instruction::Output(Mem::Const(Constant(3))), // should continue here if not jumped
            Instruction::Output(Mem::Const(Constant(3))), // should skip to this line if jumped
        ],
        vec![
            (Constant(1), Value::Int(mem)),
            (Constant(2), Value::Int(stack)),
            (Constant(3), Value::Int(1)),
        ]
    );

    match res.as_ref() {
        "1" => true,
        "11" => false,
        v => panic!(format!("test_cond_jump produced unexpected output {:?}", v)),
    }
}
