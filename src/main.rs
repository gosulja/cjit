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
        Instruction::Add, /* 15 on the stack */
        Instruction::Load(3),
        Instruction::Mul, /* 45 on the stack */
        Instruction::Load(2),
        Instruction::Sub, /* 43 on the stack */
        Instruction::Ret
    ]);

    println!("[Example 2] Result: {}", invoker.execute(&*compiler.compile(&test2)));

    println!("[Example 3] duplicate and swap on the stack: load 42, dupe it, load 10, swap them -> [42, 10, 42] -> add -> mul -> 2184");
    let test3 = Program::new(vec![
        Instruction::Load(42),
        Instruction::Dup, /* [42, 42] */
        Instruction::Load(10), /* [42, 42, 10] */
        Instruction::Swap, /* [42, 10, 42] */
        Instruction::Add, /* [42, 52] */
        Instruction::Mul, /* [2184] */
        Instruction::Ret
    ]);

    println!("[Example 3] Result: {}", invoker.execute(&*compiler.compile(&test3)));

    println!("[Example 3] storing and loading variables");
    println!("[Example 3] load 25 and 17 into variables, load the variables and add them");
    let test4 = Program::new(vec![
        Instruction::Load(25),
        Instruction::Store(0), /* 25 -> 0 [variable 0] */
        Instruction::Load(17),
        Instruction::Store(1), /* 17 -> 1 [variable 1] */
        Instruction::LoadVar(0), /* 25 <- 0 [from variable 0, load] */
        Instruction::LoadVar(1), /* 17 <- 1 [from variable 1, load] */
        Instruction::Add, /* Add the two stored and loaded variables */
        Instruction::Ret
    ]);

    println!("[Example 3] Result: {}", invoker.execute(&*compiler.compile(&test4)));

    println!("[Example 4] bitwise operations: (5 << 2) | (3 & 7)");
    let test5 = Program::new(vec![
        Instruction::Load(5),
        Instruction::Load(2),
        Instruction::Shl, /* 5 << 2   -> [20] */
        Instruction::Load(3),
        Instruction::Load(7),
        Instruction::Band, /* 3 & 7    -> [3] */
        Instruction::Bor, /* 20 | 3   -> [23] */
        Instruction::Ret
    ]);

    println!("[Example 4] Result: {}", invoker.execute(&*compiler.compile(&test5)));

    println!("[Example 5] JmpIfNot test with 0 (should jump)");
    let jump_test_zero = Program::new(vec![
        Instruction::Load(0),           /* load 0 to represent false*/
        Instruction::JmpIfNot(1),
        Instruction::Load(999),
        Instruction::Label(1),
        Instruction::Load(42),
        Instruction::Ret               /* Should return 42 */
    ]);
    println!("[Example 5] Result: {}", invoker.execute(&*compiler.compile(&jump_test_zero)));

    println!("[Example 6] JmpIfNot test with 1 (should not jump)");
    let jump_test_one = Program::new(vec![
        Instruction::Load(1), /* load 1 to represent true */
        Instruction::JmpIfNot(1),
        Instruction::Load(999),
        Instruction::Label(1),
        Instruction::Load(42),
        Instruction::Add, /* 999 + 42 = 1041 */
        Instruction::Ret
    ]);
    println!("[Example 6] Result (should be 1041): {}", invoker.execute(&*compiler.compile(&jump_test_one)));

    println!("[Example 7] Simple comparison test: 1 <= 5");
    let cmp_test = Program::new(vec![
        Instruction::Load(1),
        Instruction::Load(5),
        Instruction::Lte, /* push 1 */
        Instruction::Ret
    ]);
    println!("[Example 7] Result: {}", invoker.execute(&*compiler.compile(&cmp_test)));

    println!("[Example 8] Loop from 0 to 10");
    let loop_test = Program::new(vec![
        /*
            i = 0
        */
        Instruction::Load(0),
        Instruction::Store(0),

        /* Start */
        Instruction::Label(1),

        /*
            if i >= 10 then goto label 2
        */
        Instruction::LoadVar(0),    /* i */
        Instruction::Load(10),      /* [load] 10 */
        Instruction::Gte,           /* i >= 10 */
        Instruction::JmpIf(2),      /* true? jump to label 2 */

        /* Body */
        Instruction::LoadVar(0),    /* i */
        Instruction::Load(1),       /* 1 */
        Instruction::Add,           /* i + 1 */
        Instruction::Store(0),      /* i = i + 1 */
        Instruction::Jmp(1),

        /* End */
        Instruction::Label(2),
        Instruction::LoadVar(0),
        Instruction::Ret, /* Return i */
    ]);

    println!("[Example 8] Result: {}", invoker.execute(&*compiler.compile(&loop_test)));
}
