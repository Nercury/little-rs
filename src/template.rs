use {
    Constant,
    Parameter,
    Call,
    Instruction,
    Options,
    OptionsTemplate,
};

/// All the data required to load the processor.
pub struct Template<V> {
    pub constants: Options<Constant, V>,
    pub parameters_template: OptionsTemplate<Parameter>,
    pub calls_template: OptionsTemplate<Call>,
    pub instructions: Vec<Instruction>,
    pub bindings_capacity: u32,
}

impl<V> Template<V> {
    pub fn new(
        constants: Options<Constant, V>,
        parameters_template: OptionsTemplate<Parameter>,
        calls_template: OptionsTemplate<Call>,
        instructions: Vec<Instruction>,
        bindings_capacity: u32,
    ) -> Template<V> {
        Template {
            constants: constants,
            parameters_template: parameters_template,
            calls_template: calls_template,
            instructions: instructions,
            bindings_capacity: bindings_capacity,
        }
    }

    pub fn empty() -> Template<V> {
        Template {
            constants: Options::empty(),
            parameters_template: OptionsTemplate::empty(),
            calls_template: OptionsTemplate::empty(),
            instructions: vec![],
            bindings_capacity: 0,
        }
    }

    pub fn push_constant(mut self, index: Constant, value: V) -> Self {
        self.constants.push(index, value);
        self
    }

    pub fn push_parameter<S: Into<String>>(mut self, key: S, index: Parameter) -> Self {
        self.parameters_template.push(key, index);
        self
    }

    pub fn push_call<S: Into<String>>(mut self, key: S, index: Call) -> Self {
        self.calls_template.push(key, index);
        self
    }

    pub fn push_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    pub fn push_instructions<I: IntoIterator<Item=Instruction>>(mut self, instructions: I) -> Self {
        self.instructions.extend(instructions.into_iter());
        self
    }
}
