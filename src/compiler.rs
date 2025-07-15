use std::mem;
use std::collections::HashMap;

/*
    Targeting x86-64
*/

/*
    A simple bytecode instruction set.
    Since this is a proof of concept JIT compiler,
    only a small instruction set is implemented.
*/
#[derive(Debug, Clone)]
pub enum Instruction {
    Load(i64),      /* Load an immediate value onto the stack :D */
    Add,            /* Self explanatory, stack based, pop values, add them, push result */
    Sub,            /* Self explanatory, stack based, pop values, sub them, push result */
    Mul,            /* Self explanatory, stack based, pop values, mul them, push result */
    Div,            /* Self explanatory, stack based, pop values, div them, push result */
    Write,          /* Write the top of the stack value to the console */
    Ret,            /* Return */
}

/*
    A structure to represent a program in the form of bytecode instructions.
*/
pub struct Program {
    insts: Vec<Instruction>,
}

impl Program {
    pub fn new(insts: Vec<Instruction>) -> Self {
        Self { insts }
    }
}

/*
    A structure to represent a compiler instance, containing the bytecode and stack offset.
*/
pub struct Compiler {
    bytecode: Vec<u8>,      /* Bytecode, a "string" of bytes to represent code */
    stk_offset: i32,        /* Stack Offset */
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            bytecode: vec![],
            stk_offset: 0,
        }
    }

    /*
        Emit a byte to the bytecode buffer.
    */
    fn emit(&mut self, bytes: &[u8]) {
        self.bytecode.extend_from_slice(bytes); /* Bytes cloned? Let's use extend_from_slice */
    }

    /*
        Emit function prologue, set up the function.
    */
    fn emit_fn_prologue(&mut self) {
        self.emit(&[0x55]);                 /* push rbp */
        self.emit(&[0x48,0x89,0xE5]);       /* mov rbp, rsp */
        self.emit(&[                        /* sub rsp, 1024 ; allocate enough stack space */
            0x48,0x81,0xEC,0x00,
            0x04,0x00,0x00
        ]);
    }

    /*
        Emit function epilogue, "end" the function.
    */
    fn emit_fn_epilogue(&mut self) {
        self.emit(&[0x48,0x89,0xEC]);       /* mov rsp, rbp */
        self.emit(&[0x5D]);                 /* pop rbp */
        self.emit(&[0xC3]);                 /* ret */
    }

    /*
        Load an immediate value (value included within the opcode,
        not to be confused with values in a register or memory)
    */
    fn emit_load_imm(&mut self, val: i64) {
        self.emit(&[0x48,0xB8]);            /* mov rax, <val> */
        self.emit(&val.to_le_bytes());            /* Convert the immediate value into bytes and emit them */
        self.emit(&[0x50]);                 /* push rax */
        self.stk_offset += 8;                     /* Increment the stack offset by 8 */
    }

    /*
        Perform a binary operation, like add, sub, mul, div.
        Pop the two values and push the result.
    */
    fn emit_binop(&mut self, op: &str) {
        self.emit(&[0x5B]);                 /* pop rbx  ; second */
        self.emit(&[0x58]);                 /* pop rax  ; first */

        match op {
            "add" => self.emit(&[0x48,0x01,0xD8]),          /* add rax, rbx */
            "sub" => self.emit(&[0x48,0x29,0xD8]),          /* sub rax, rbx */
            "mul" => self.emit(&[0x48,0x0F,0xAF,0xC3]),     /* imul rax, rbx */
            /*
                Division is a bit more complex, we need
                to sign extend rax to rbx:rax, which
                can be done with cqo instruction.

                https://www.felixcloutier.com/x86/cwd:cdq:cqo
            */
            "div" => {
                self.emit(&[0x48,0x99]);                    /* cqo */
                self.emit(&[0x48,0xF7,0xFb]);               /* idiv rbx */
            }

            _ => panic!("unknown binop op: {}", op),
        }

        self.emit(&[0x50]);     /* push rax ; result */
        self.stk_offset -= 8;         /* We just did two pops, so one push */
    }

    pub fn compile(&mut self, program: &Program) -> Vec<u8> {
        self.emit_fn_prologue();

        for i in &program.insts {
            match i {
                Instruction::Load(v) => self.emit_load_imm(*v),
                Instruction::Add => self.emit_binop("add"),
                Instruction::Sub => self.emit_binop("sub"),
                Instruction::Mul => self.emit_binop("mul"),
                Instruction::Div => self.emit_binop("div"),
                Instruction::Ret => { self.emit(&[0x58]); break; }  /* pop rax */
                Instruction::Write => {}
            }
        }

        self.emit_fn_epilogue();
        self.bytecode.clone()
    }

}

/*
    A structure to represent an invoker, this will handle the execution of the bytecode generated from the compiler.
*/
pub struct Invoker;

impl Invoker {
    pub fn new() -> Self {
        Self
    }

    /*
        Finally, execute the code from the compiler.
    */
    pub fn execute(&mut self, code: &[u8]) -> i64 {
        unsafe {
            /*
                Allocate memory which is executable
            */
            let psize = 4096;       /* Page size */
            let csize = (code.len() + psize - 1) & !(psize - 1);    /* Calculate the size of the generate code */

            /*
                Pointer to the allocated memory
            */
            let p = libc::mmap(
                std::ptr::null_mut(),
                csize,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0
            );

            if p == libc::MAP_FAILED {
                panic!("failed to allocate memory for Invoker.");
            }

            /*
                Then copy the code to memory whichi is executable
            */
            std::ptr::copy_nonoverlapping(code.as_ptr(), p as *mut u8, code.len());

            /*
                And then obviously, make it executable.
            */
            if libc::mprotect(p, csize, libc::PROT_READ | libc::PROT_EXEC) != 0 {
                panic!("failed to make memory executable for Invoker.");
            }

            /*
                Cast the function pointer from the executable memory
                and then, at last, execute.
            */
            let f: extern "C" fn() -> i64 = std::mem::transmute(p);
            let ret = f();

            /*
                Cleanup allocated memory
            */
            libc::munmap(p, csize);

            /* Return the result of the executed function */
            ret
        }
    }
}
