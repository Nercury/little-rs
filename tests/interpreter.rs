extern crate little;

mod mock;

use std::collections::HashMap;
use std::io::Read;
use std::error::Error;

use little::*;
use little::interpreter::Interpreter;

use mock::Value;

#[test]
fn error_if_missing_constant() {
    let funs = HashMap::new();
    let mut i = Interpreter::new();
    let p = i.build_processor(
        Template::empty()
            .push_instructions(vec![
                Instruction::Output { location: Mem::Const(Constant(1)) },
            ]),
        &funs
    ).unwrap();

    let mut res = String::new();

    let res = p.run(Value::Null)
        .read_to_string(&mut res)
        .err()
        .expect("expected to receive error from read");

    assert_eq!("constant is missing", res.description());
}

#[test]
fn can_handle_interupt() {
    let funs = HashMap::new();
    let mut i = Interpreter::new();
    let p = i.build_processor(
        Template::empty()
            .push_constant(Constant(1), Value::Str("Abr".into()))
            .push_instructions(vec![
                Instruction::Output { location: Mem::Const(Constant(1)) },
                Instruction::Interupt,
                Instruction::Output { location: Mem::Const(Constant(1)) },
            ]),
        &funs
    ).unwrap();

    let mut res = String::new();
    let mut received_interupt = false;

    let mut interpreter = p.run(Value::Null);
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
                Instruction::Output { location: Mem::Const(Constant(1)) }
            ]),
        &funs
    ).unwrap();

    let mut res = String::new();

    let res = p.run(Value::Null)
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
                Instruction::Pop { times: 1 }
            ]),
        &funs
    ).unwrap();

    let mut res = String::new();

    let res = p.run(Value::Null)
        .read_to_string(&mut res)
        .err()
        .expect("expected to receive error from read");

    assert_eq!("stack underflow", res.description());
}

#[test]
fn exit() {
    let res = from_instructions_and_params(Vec::new(), Value::Null);
    assert_eq!("", res);
}

#[test]
fn output_params() {
    let res = from_instructions_and_params(
        vec![
            Instruction::Output { location: Mem::Parameters }
        ],
        Value::Str("Hello".into())
    );

    assert_eq!("Hello", res);
}

#[test]
fn should_jump() {
    let res = from_instructions_and_constants(
        vec![
            Instruction::Output { location: Mem::Const(Constant(1)) },
            Instruction::Jump { pc: 3 },
            Instruction::Output { location: Mem::Const(Constant(2)) },
            Instruction::Output { location: Mem::Const(Constant(3)) },
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
            Instruction::Output { location: Mem::Const(Constant(1)) }
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
                Instruction::Push { location: Mem::Const(Constant(1)) },
                Instruction::Push { location: Mem::Const(Constant(2)) },
                Instruction::Call { call: Call(1), argc: 2, push_result_to_stack: true },
                Instruction::Output { location: Mem::StackTop1 },
            ]),
        &funs
    ).unwrap();

    let mut res = String::new();

    p.run(Value::Null)
        .read_to_string(&mut res)
        .unwrap();

    assert_eq!("5", &res);
}

#[test]
fn push_const_output_stack_top1() {
    let res = from_instructions_and_constants(
        vec![
            Instruction::Push { location: Mem::Const(Constant(1)) },
            Instruction::Output { location: Mem::StackTop1 },
        ],
        vec![
            (Constant(1), Value::Str("Hello Stack 1".into()))
        ]
    );

    assert_eq!("Hello Stack 1", res);
}

#[test]
fn push_constants_output_stack_top2() {
    let res = from_instructions_and_constants(
        vec![
            Instruction::Push { location: Mem::Const(Constant(2)) },
            Instruction::Push { location: Mem::Const(Constant(1)) },
            Instruction::Output { location: Mem::StackTop2 },
        ],
        vec![
            (Constant(1), Value::Str("Do not show this".into())),
            (Constant(2), Value::Str("Hello Stack 2".into())),
        ]
    );

    assert_eq!("Hello Stack 2", res);
}

#[test]
fn load_binding_from_const_output_binding() {
    let res = from_instructions_and_constants(
        vec![
            Instruction::Load { binding: Binding(2), location: Mem::Const(Constant(1)) },
            Instruction::Output { location: Mem::Binding(Binding(2)) },
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
            Instruction::Load { binding: Binding(0), location: Mem::Parameters },
            Instruction::Output { location: Mem::Binding(Binding(0)) },
        ],
        Value::Str("Hello Binding".into())
    );

    assert_eq!("Hello Binding", res);
}

#[test]
fn load_binding_from_binding_stack1_stack2_output3() {
    let res = from_instructions_and_constants(
        vec![
            Instruction::Load { binding: Binding(0), location: Mem::Const(Constant(1)) },
            Instruction::Load { binding: Binding(2), location: Mem::Const(Constant(2)) },
            Instruction::Load { binding: Binding(1), location: Mem::Binding(Binding(0)) },
            Instruction::Push { location: Mem::Binding(Binding(2)) },
            Instruction::Push { location: Mem::Binding(Binding(1)) },
            Instruction::Load { binding: Binding(3), location: Mem::StackTop1 },
            Instruction::Load { binding: Binding(4), location: Mem::StackTop2 },
            Instruction::Output { location: Mem::StackTop1 },
            Instruction::Output { location: Mem::StackTop2 },
        ],
        vec![
            (Constant(1), Value::Str("Hello".into())),
            (Constant(2), Value::Str("World".into())),
        ]
    );

    assert_eq!("HelloWorld", res);
}

#[test]
fn push_from_stack_to_stack() {
    let res = from_instructions_and_constants(
        vec![
            Instruction::Push { location: Mem::Const(Constant(1)) },
            Instruction::Push { location: Mem::Const(Constant(2)) },
            Instruction::Push { location: Mem::StackTop1 },
            Instruction::Push { location: Mem::StackTop2 },
            Instruction::Output { location: Mem::StackTop1 },
            Instruction::Output { location: Mem::StackTop2 },
        ],
        vec![
            (Constant(1), Value::Str("Hello".into())),
            (Constant(2), Value::Str("World".into())),
        ]
    );

    assert_eq!("WorldWorld", res);
}

#[test]
fn output_constant_twice() {
    let res = from_instructions_and_constants(
        vec![
            Instruction::Output { location: Mem::Const(Constant(1)) },
            Instruction::Output { location: Mem::Const(Constant(1)) },
        ],
        vec![
            (Constant(1), Value::Str("Hello".into())),
        ]
    );

    assert_eq!("HelloHello", res);
}

#[test]
fn output_different_constants() {
    let res = from_instructions_and_constants(
        vec![
            Instruction::Output { location: Mem::Const(Constant(1)) },
            Instruction::Output { location: Mem::Const(Constant(3)) },
            Instruction::Output { location: Mem::Const(Constant(2)) },
        ],
        vec![
            (Constant(1), Value::Str("Hello".into())),
            (Constant(2), Value::Str("World".into())),
            (Constant(3), Value::Str(" ".into())),
        ]
    );

    assert_eq!("Hello World", res);
}

fn from_instructions_and_params(
    instructions: Vec<Instruction>,
    params: Value
) -> String {
    let funs = HashMap::new();
    let mut i = Interpreter::new();
    let p = i.build_processor(
        Template::empty()
            .push_instructions(instructions),
        &funs
    ).unwrap();

    let mut res = String::new();

    p.run(params)
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

    p.run(Value::Null)
        .read_to_string(&mut res)
        .unwrap();

    res
}

/// Check if stack compared to mem using condition produces a jump.
fn test_cond_jump(stack: i64, mem: i64, cond: Cond) -> bool {
    let res = from_instructions_and_constants(
        vec![
            Instruction::Push { location: Mem::Const(Constant(2)) },
            Instruction::CondJump { pc: 3, location: Mem::Const(Constant(1)), test: cond },
            Instruction::Output { location: Mem::Const(Constant(3)) }, // should continue here if not jumped
            Instruction::Output { location: Mem::Const(Constant(3)) }, // should skip to this line if jumped
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
