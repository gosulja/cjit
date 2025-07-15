use crate::compiler::{Compiler, Instruction, Invoker, Program};

mod compiler;

fn main() {
    let mut compiler = Compiler::new();
    let mut invoker = Invoker::new();

    /* 100 + 200 */
    let test = Program::new(vec![
        Instruction::Load(100),
        Instruction::Load(200),
        Instruction::Add,
        Instruction::Ret
    ]);

    let bytecode = compiler.compile(&test);
    let result = invoker.execute(&*bytecode);

    println!("Result: {}", result);
}
