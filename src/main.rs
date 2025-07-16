use crate::compiler::{Compiler, Instruction, Invoker, Program};

mod compiler;

fn main() {
    let mut compiler = Compiler::new();
    let mut invoker = Invoker::new();

    println!("[Example 1] 100 + 200");
    let test = Program::new(vec![
        Instruction::Load(100),
        Instruction::Load(200),
        Instruction::Add,
        Instruction::Ret
    ]);

    println!("[Example 1] Result: {}", invoker.execute(&*compiler.compile(&test)));

    println!("[Example 2] (10 + 5) * 3 - 2");
    let test2 = Program::new(vec![
        Instruction::Load(10),
        Instruction::Load(5),
        Instruction::Add,           /* 15 on the stack */
        Instruction::Load(3),
        Instruction::Mul,           /* 45 on the stack */
        Instruction::Load(2),
        Instruction::Sub,           /* 43 on the stack */
        Instruction::Ret
    ]);

    println!("[Example 2] Result: {}", invoker.execute(&*compiler.compile(&test2)));

    println!("[Example 3] duplicate and swap on the stack: load 42, dupe it, load 10, swap them -> [42, 10, 42] -> add -> mul -> 2184");
    let test3 = Program::new(vec![
        Instruction::Load(42),
        Instruction::Dup,           /* [42, 42] */
        Instruction::Load(10),      /* [42, 42, 10] */
        Instruction::Swap,          /* [42, 10, 42] */
        Instruction::Add,           /* [42, 52] */
        Instruction::Mul,           /* [2184] */
        Instruction::Ret
    ]);

    println!("[Example 3] Result: {}", invoker.execute(&*compiler.compile(&test3)));

    println!("[Example 3] storing and loading variables");
    println!("[Example 3] load 25 and 17 into variables, load the variables and add them");
    let test4 = Program::new(vec![
        Instruction::Load(25),
        Instruction::Store(0),      /* 25 -> 0 [variable 0] */
        Instruction::Load(17),
        Instruction::Store(1),      /* 17 -> 1 [variable 1] */
        Instruction::LoadVar(0),    /* 25 <- 0 [from variable 0, load] */
        Instruction::LoadVar(1),    /* 17 <- 1 [from variable 1, load] */
        Instruction::Add,           /* Add the two stored and loaded variables */
        Instruction::Ret
    ]);

    println!("[Example 3] Result: {}", invoker.execute(&*compiler.compile(&test4)));

}
