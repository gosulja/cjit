# cjit
A Proof of Concept JIT Compiler

This includes an incredibly small instruction set consisting of basic arithmetic operators such as ADD, SUB, MUL, and DIV.
This depends on `libc` to allocate executable memory, and execute it within the process.

What is a JIT Compiler? According to Wikipedia, a JIT compiler is compilation of computer code during execution of a program rather than before execution. This commonly consists of bytecode translation to machine code, which is executed directly.

JIT compilation is a combination of the two traditional approaches to translation to machine code: AOT (ahead of time compilation) and interpretation, which combines some advantages and drawbacks of both. Roughly, JIT compilation combines the speed of compiled code with the flexibility of interpretation, with the overhead of an interpreter and the additional overhead of compiling and linking. JIT compilation is a form of dynamic compilation, and allows adaptive optimization such as dynamic recompilation and microarchitecture specific speedups.
