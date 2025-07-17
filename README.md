# cjit
A Proof of Concept JIT Compiler

This includes an incredibly small instruction set consisting of basic arithmetic operators such as ADD, SUB, MUL, and DIV.
This depends on `libc` to allocate executable memory, and execute it within the process.

What is a JIT Compiler? According to Wikipedia, a JIT compiler is compilation of computer code during execution of a program rather than before execution. This commonly consists of bytecode translation to machine code, which is executed directly.

JIT compilation is a combination of the two traditional approaches to translation to machine code: AOT (ahead of time compilation) and interpretation, which combines some advantages and drawbacks of both. Roughly, JIT compilation combines the speed of compiled code with the flexibility of interpretation, with the overhead of an interpreter and the additional overhead of compiling and linking. JIT compilation is a form of dynamic compilation, and allows adaptive optimization such as dynamic recompilation and microarchitecture specific speedups.

# Examples
```rust
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
```

# Examples Results
[Example 1] 100 + 200
[Example 1] Result: 300
[Example 2] (10 + 5) * 3 - 2
[Example 2] Result: 43
[Example 3] duplicate and swap on the stack: load 42, dupe it, load 10, swap them -> [42, 10, 42] -> add -> mul -> 2184
[Example 3] Result: 2184
[Example 3] storing and loading variables
[Example 3] load 25 and 17 into variables, load the variables and add them
[Example 3] Result: 42
[Example 4] bitwise operations: (5 << 2) | (3 & 7)
[Example 4] Result: 23
[Example 5] JmpIfNot test with 0 (should jump)
[Example 5] Result: 42
[Example 6] JmpIfNot test with 1 (should not jump)
[Example 6] Result (should be 1041): 1041
[Example 7] Simple comparison test: 1 <= 5
[Example 7] Result: 1
[Example 8] Loop from 0 to 10
[Example 8] Result: 10
