use std::mem;
use std::collections::HashMap;

/*
    Targeting x86-64
*/

/*
    Updated Instruction set
*/
#[derive(Debug, Clone)]
pub enum Instruction {
    Load(i64),      /* Load an immediate value onto the stack :D */
    Dup,            /* Duplicate stack top value */
    Pop,            /* Pop stack top value */
    Swap,           /* Swap stack two top value */
    Add,            /* Self explanatory, stack based, pop values, add them, push result */
    Sub,            /* Self explanatory, stack based, pop values, sub them, push result */
    Mul,            /* Self explanatory, stack based, pop values, mul them, push result */
    Div,            /* Self explanatory, stack based, pop values, div them, push result */
    Mod,            /* Self explanatory, stack based, pop values, mod them, push result */
    Neg,            /* Negate the stack top value */
    Eq,             /* Comparison, == */
    Ne,             /* Comparison, != */
    Lt,             /* Comparison, < */
    Gt,             /* Comparison, > */
    Lte,            /* Comparison, <= */
    Gte,            /* Comparison, >= */
    And,            /* Logical, && */
    Or,             /* Logical, || */
    Not,            /* Logical, ! or NOT */
    Band,           /* Bit, and */
    Bor,            /* Bit, or */
    Bxor,           /* Bit, xor */
    Bnot,           /* Bit, not */
    Shl,            /* Bit, shift left */
    Shr,            /* Bit, shift right */
    Store(u32),     /* Variable, store top of stack to a labelled local variable */
    LoadVar(u32),   /* Variable, load labelled local variable value to stack */
    Jmp(u32),       /* Conditional, an unconditional jump to a label */
    JmpIf(u32),     /* Conditional, a conditional jump to a label, condition is true if top stack value is not zero */
    JmpIfNot(u32),  /* Conditional, a conditional jump to a label, condition is true if top stack value is zero */
    Label(u32),     /* Control Flow?, define a label for jumps, funcstions etc */
    Call(u32),      /* Control Flow, call a function via label */
    Write,          /* Write the top of the stack value to the console */
    WriteChar,      /* Write the top of the stack value as a char */
    Read,           /* read an int from stdin */
    Ret,            /* Return from function */
    Halt,           /* Stop execution */
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
    bytecode: Vec<u8>,                  /* Bytecode, a "string" of bytes to represent code */
    stk_offset: i32,                    /* Stack Offset */
    labels: HashMap<u32, usize>,        /* Map labels to position in bytecode, for example label 1 could be mapped to position 56 in the bytecode */
    label_patches: Vec<(usize, u32)>    /* All patches needed for labels for forward jmps */
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            bytecode: vec![],
            stk_offset: 0,
            labels: HashMap::new(),
            label_patches: vec![],
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
        Perform a duplication of the top stack value
    */
    fn emit_dup(&mut self) {
        self.emit(&[0x48,0x8B,0x04,0x24]);  /* mov rax, [rsp] */
        self.emit(&[0x50]);                 /* push rax */
        self.stk_offset += 8;
    }

    /*
        Pop top stack value (simple)
    */
    fn emit_pop(&mut self) {
        self.emit(&[0x58]);                 /* pop rax */
        self.stk_offset -= 8;
    }

    /*
        Simply swap two top stack values
    */
    fn emit_swap(&mut self) {
        self.emit(&[0x5B]);                 /* pop rbx */
        self.emit(&[0x58]);                 /* pop rax */
        self.emit(&[0x53]);                 /* push rbx */
        self.emit(&[0x50]);                 /* push rax */
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
            "and" => self.emit(&[0x48,0x21,0xD8]),          /* and rax, rbx */
            "or" => self.emit(&[0x48,0x09,0xD8]),           /* or rax, rbx */
            "xor" => self.emit(&[0x48,0x31,0xD8]),          /* xor rax, rbx */
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

            "mod" => {
                self.emit(&[0x48,0x99]);                    /* cqo */
                self.emit(&[0x48,0xF7,0xFb]);               /* idiv rbx */
                self.emit(&[0x48,0x89,0xD0]);               /* mov rax, rdx  ; for remainder */
            }

            "shl" => {
                self.emit(&[0x48,0x89,0xD9]);               /* mov rcx, rbx */
                self.emit(&[0x48,0xD3,0xE0]);               /* shl rax, cl */
            }

            "shr" => {
                self.emit(&[0x48,0x89,0xD9]);               /* mov rcx, rbx */
                self.emit(&[0x48,0xD3,0xF8]);               /* sar rax, cl */
            }

            _ => panic!("unknown binop op: {}", op),
        }

        self.emit(&[0x50]);     /* push rax ; result */
        self.stk_offset -= 8;         /* We just did two pops, so one push */
    }

    /*
        Perform a comparison, like eq, ne, lt.
        Push the result
    */
    fn emit_cmp(&mut self, op: &str) {
        self.emit(&[0x5B]);             /* pop rbx  ;   second */
        self.emit(&[0x58]);             /* pop rax  ;   first */
        self.emit(&[0x48,0x39,0xD8]);   /* cmp rax, rbx */
        self.emit(&[0x48,0x31,0xC0]);   /* xor rax, rax     ; clear */

        match op {
            "eq"  => self.emit(&[0x0F,0x94,0xC0]),   /* sete al */
            "ne"  => self.emit(&[0x0F,0x95,0xC0]),   /* setne al */
            "lt"  => self.emit(&[0x0F,0x9C,0xC0]),   /* setl al */
            "lte" => self.emit(&[0x0F,0x9E,0xC0]),   /* setle al */
            "gt"  => self.emit(&[0x0F,0x9F,0xC0]),   /* setg al */
            "gte" => self.emit(&[0x0F,0x9D,0xC0]),   /* setge al */
            _ => panic!("unknown cmp op: {}", op),
        }

        self.emit(&[0x50]);             /* push rax */
        self.stk_offset -= 8;
    }

    /*
        Perform a unary, like not, neg, and bit not.
        Push the result
    */
    fn emit_unary(&mut self, op: &str) {
        self.emit(&[0x58]);             /* pop rax  ;   operand */

        match op {
            "bnot" => self.emit(&[0x48,0xF7,0xD0]),   /* not rax */
            "neg"  => self.emit(&[0x48,0xF7,0xD8]),   /* neg rax */
            "ne"  => {
                self.emit(&[0x48,0x85,0xC0]);         /* test rax, rax */
                self.emit(&[0x48,0x31,0xC0]);         /* xor rax, rax */
                self.emit(&[0x0F,0x94,0xC0]);         /* sete al */
            }
            _ => panic!("unknown cmp op: {}", op),
        }

        self.emit(&[0x50]);             /* push rax */
        self.stk_offset -= 8;
    }

    /*
        Store top of tack to variable
        `id` being the number assigned to a string literal representing the variable name,
        this should be handled when generating bytecode after or during parsing.
    */
    fn emit_store(&mut self, id: u32) {
        let offset = (id as i32 + 1) * 8;            /* Gonna use RBP offset, so calculate it */
        self.emit(&[0x58]);                         /* pop rax */
        self.emit(&[0x48,0x89,0x85]);               /* mov [rbp-offset], rax */
        self.emit(&(-(offset as i32)).to_le_bytes());      /* Convert the offset into bytes */
        self.stk_offset -= 8;
    }

    /*
        Load the local variable from the `id` onto tjhe stack
    */
    fn emit_load_var(&mut self, id: u32) {
        let offset = (id as i32 + 1) * 8;            /* Gonna use RBP offset, so calculate it */
        self.emit(&[0x48,0x8B,0x85]);               /* mov rax, [rbp-offset] */
        self.emit(&(-(offset as i32)).to_le_bytes());     /* Convert the offset into bytes */
        self.emit(&[0x50]);                         /* push rax */
        self.stk_offset += 8;
    }

    /*
        Emit a jump
    */
    fn emit_jmp(&mut self, label: u32, is_conditional_jmp: Option<bool>) {
        match is_conditional_jmp {
            None => {
                /*
                    Emit an unconditional "jmp"
                */
                self.emit(&[0xE9]);                     /* jmp rel32 */
                let pos = self.bytecode.len();          /* Calculate the patch position in the bytecode */
                self.label_patches.push((pos, label));  /* Push the label ID along with it's patch position */
                self.emit(&[0x00,0x00,0x00,0x00]);
            }

            Some(true) => {
                /*
                    Emit a condition jump, so if top of stack value is not zero (correction: non-zero)
                */
                self.emit(&[0x58]);                 /* pop rax */
                self.emit(&[0x48,0x85,0xC0]);       /* test rax, rax */
                self.emit(&[0x0F,0x85]);            /* jnz rel32 */
                let pos = self.bytecode.len();      /* Calculate the patch position in the bytecode */
                self.label_patches.push((pos, label));
                self.emit(&[0x00,0x00,0x00,0x00]);
                self.stk_offset -= 8;
            }

            Some(false) => {
                /*
                    Emit a condition jump, so if top of stack value is zero
                */
                self.emit(&[0x58]);                 /* pop rax */
                self.emit(&[0x48,0x85,0xC0]);       /* test rax, rax */
                self.emit(&[0x0F,0x84]);            /* jz rel32 */
                let pos = self.bytecode.len();      /* Calculate the patch position in the bytecode */
                self.label_patches.push((pos, label));
                self.emit(&[0x00,0x00,0x00,0x00]);
                self.stk_offset -= 8;
            }
        }
    }

    /*
        Write syscall
    */
    fn emit_write(&mut self) {
        /* TODO: implement proper syscall calling, but right now just return it */
        self.emit(&[0x58]); /* pop rax  ; write this value */
        self.emit(&[0x50]); /* push rax  ; write this value */
    }

    /*
        Helper for patching jumps
    */
    fn patch_jumps(&mut self) {
        for (pos, label) in &self.label_patches {
            if let Some(&p) = self.labels.get(label) {
                let offset = p as i32 - (*pos as i32 + 4);  /* Calculate offset of the patch position */
                let bytes = offset.to_le_bytes();
                for (i, &b) in bytes.iter().enumerate() {
                    self.bytecode[pos + i] = b; /* Patch jumps */
                }
            }
        }
    }

    pub fn compile(&mut self, program: &Program) -> Vec<u8> {
        self.bytecode.clear();
        self.emit_fn_prologue();

        /*
            Initially, collect labels.
        */
        for (i, inst) in program.insts.iter().enumerate() {
            if let Instruction::Label(id) = inst {
                self.labels.insert(*id, self.bytecode.len());
            }
        }

        /*
            After, compile instructions
        */
        for i in &program.insts {
            match i {
                Instruction::Load(v) => self.emit_load_imm(*v),
                Instruction::Dup => self.emit_dup(),
                Instruction::Pop => self.emit_pop(),
                Instruction::Swap => self.emit_swap(),
                Instruction::Add => self.emit_binop("add"),
                Instruction::Sub => self.emit_binop("sub"),
                Instruction::Mul => self.emit_binop("mul"),
                Instruction::Div => self.emit_binop("div"),
                Instruction::Mod => self.emit_binop("mod"),
                Instruction::Neg => self.emit_unary("neg"),
                Instruction::Eq => self.emit_cmp("eq"),
                Instruction::Ne => self.emit_cmp("ne"),
                Instruction::Lt => self.emit_cmp("lt"),
                Instruction::Lte => self.emit_cmp("le"),
                Instruction::Gt => self.emit_cmp("gt"),
                Instruction::Gte => self.emit_cmp("ge"),
                Instruction::And => self.emit_binop("and"),
                Instruction::Or => self.emit_binop("or"),
                Instruction::Not => self.emit_unary("not"),
                Instruction::Band => self.emit_binop("and"),
                Instruction::Bor => self.emit_binop("or"),
                Instruction::Bxor => self.emit_binop("xor"),
                Instruction::Bnot => self.emit_unary("bnot"),
                Instruction::Shl => self.emit_binop("shl"),
                Instruction::Shr => self.emit_binop("shr"),
                Instruction::Store(var_id) => self.emit_store(*var_id),
                Instruction::LoadVar(var_id) => self.emit_load_var(*var_id),
                Instruction::Jmp(label) => self.emit_jmp(*label, None),
                Instruction::JmpIf(label) => self.emit_jmp(*label, Some(true)),
                Instruction::JmpIfNot(label) => self.emit_jmp(*label, Some(false)),
                Instruction::Label(_) => {},
                Instruction::Write => self.emit_write(),
                Instruction::WriteChar => self.emit_write(),
                Instruction::Read => {},
                Instruction::Call(_) => {},
                Instruction::Ret => {
                    self.emit(&[0x58]); /* pop rax */
                    break;
                }
                Instruction::Halt => break,
            }
        }

        self.patch_jumps();
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
