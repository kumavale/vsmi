#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InstructionKind {
    /// Arithmetic, Logic
    ADD,   // Rd, Rs, Rt    | Rd = Rs + Rt
    ADDI,  // Rt, Rs, Imm   | Rt = Rs + Imm
    SUB,   // Rd, Rs, Rt    | Rd = Rs - Rt
    MUL,   // Rd, Rs, Rt    | Rd = Rs * Rt
    DIV,   // Rd, Rs, Rt    | Rd = Rs / Rt

    AND,   // Rd, Rs, Rt    | Rd = Rs & Rt
    ANDI,  // Rt, Rs, Imm   | Rt = Rs & Imm
    OR,    // Rd, Rs, Rt    | Rd = Rs | Rt
    ORI,   // Rt, Rs, Imm   | Rt = Rs | Imm
    XOR,   // Rd, Rs, Rt    | Rd = Rs ^ Rt
    XORI,  // Rt, Rs, Imm   | Rt = Rs ^ Imm

    /// Constant
    LI,    // Rd, Imm       | Rd = Imm
    LUI,   // Rt, Imm       | Rt[31:16] = Imm

    /// Comparison

    /// Branch
    //B,     // label         | goto label
    BEQ,   // Rs, Rt, label | goto label if Rs == Rt
    //BGE,   // Rs, Rt, label | goto label if Rs >= Rt
    //BGT,   // Rs, Rt, label | goto label if Rs > Rt
    BLE,   // Rs, Rt, label | goto label if Rs <= Rt
    BLT,   // Rs, Rt, label | goto label if Rs < Rt
    //BNEZ,  // Rs, Rt, label | goto label if Rs != 0

    /// Jump
    J,     // Target        | goto Target
    JAL,   // Target        | $ra = next idx; goto Target
    JR,    // Rs, Rd        | Rd = next idx; goto Rs
    JALR,  // Rs            | goto Rs

    /// Load, Store
    LA,    // Rd, address   | Rt = idx(stack)
    LW,    // Rt, address   | Rt = stack[idx]
    SW,    // Rt, address   | stack[idx] = Rt

    /// Transfer
    MOVE,  // Rd, Rs        | Rd = Rs

    /// Exception, Interrupt
    SYSCALL,  //
    NOP,      // Do nothing
}

#[derive(Clone, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum RegisterKind {
    zero,                            //     0: Hard-wired to 0
    at,                              //     1: Reserved for pseudo-instructions
    v0, v1,                          //   2-3: Return values from functions
    a0, a1, a2, a3,                  //   4-7: Arguments to functions - not preserved by subprograms
    t0, t1, t2, t3, t4, t5, t6, t7,  //  8-15: Temporary data, not preserved by subprograms
    s0, s1, s2, s3, s4, s5, s6, s7,  // 16-23: Saved registers, preserved by subprograms
    t8, t9,                          // 24-25: More temporary registers, not preserved by subprograms
    k0, k1,                          // 26-27: Reserved for kernel. Do not use.
    gp,                              //    28: Global Area Pointer (base of global data segment)
    sp,                              //    29: Stack Pointer
    fp,                              //    30: Frame Pointer
    ra,                              //    31: Return Address
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    INSTRUCTION(InstructionKind),
    INTEGER(i32),                       // Immediate
    REGISTER(RegisterKind, usize),      // (_, Index)
    STACK(RegisterKind, usize, usize),  // (_, Append index)
    LABEL(String, usize),               // (Literal, Index)
    ADDRESS(String),                    // Literal
    EOL,                                // End of Line
}

#[derive(Debug)]
pub struct Tokens {
    token: Vec<(TokenKind, u32)>,  // (TokenKind, number of lines)
    idx: usize,                    // Current index
    foremost: bool,                // Foremost
    length: usize,                 // Token length
}

//pub type Token = (TokenKind, u32);

#[allow(dead_code)]
impl Tokens {
    pub fn new() -> Self {
        let token: Vec<(TokenKind, u32)> = Vec::new();
        Tokens { token, idx: 0, foremost: true, length: 0 }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn push(&mut self, kind: TokenKind, line: u32) {
        self.length += 1;
        self.token.push((kind, line));
    }

    pub fn reset(&mut self) {
        self.foremost = true;
        self.idx = 0;
    }

    pub fn consume(&mut self) -> Option<(TokenKind, u32)> {
        if self.foremost {
            self.foremost = false;
            Some(self.token[0].clone())
        } else {
            if self.idx+1 < self.length {
                self.idx += 1;
                Some(self.token[self.idx].clone())
            } else {
                None
            }
        }
    }

    pub fn get_token(&self) -> (TokenKind, u32) {
        self.token[self.idx].clone()
    }

    pub fn get_stack_capacity(&mut self) -> usize {
        let mut max_capacity = 0;
        while let Some(token) = self.consume() {
            if let TokenKind::STACK(_, _, a) = token.0 {
                if max_capacity < a {
                    max_capacity = a;
                }
            }
        }
        self.reset();
        max_capacity
    }

    pub fn goto(&mut self, idx: usize) {
        self.idx = idx;
    }

    pub fn idx(&self) -> usize {
        self.idx
    }

    /// Get index of String same as TokenKind::ADDRESS() from TokenKind::LABEL()
    pub fn expect_address(&self) -> Result<usize, String> {
        if let (TokenKind::ADDRESS(s), _) = self.token[self.idx].clone() {
            for t in &self.token {
                if let (TokenKind::LABEL(name, idx), _) = t {
                    if &*s == &*name {
                        return Ok(*idx);
                    }
                }
            }
            let (_, line) = self.token[self.idx];
            Err(format!("{}: invalid address: {}", line, s))
        } else {
            let (kind, line) = self.token[self.idx].clone();
            Err(format!("{}: expect TokenKind::ADDRESS(String). but got: {:?}", line, kind))
        }
    }

    pub fn expect_instruction(&self) -> Result<InstructionKind, String> {
        if let (TokenKind::INSTRUCTION(k), _) = self.token[self.idx] {
            Ok(k)
        } else {
            let (kind, line) = self.token[self.idx].clone();
            Err(format!("{}: expect TokenKind::INSTRUCTION(InstructionKind). but got: {:?}", line, kind))
        }
    }

    pub fn expect_register(&self) -> Result<usize, String> {
        if let (TokenKind::REGISTER(_, i), _) = self.token[self.idx] {
            Ok(i)
        } else {
            let (kind, line) = self.token[self.idx].clone();
            Err(format!("{}: expect TokenKind::REGISTER(RegisterKind, usize). but got: {:?}", line, kind))
        }
    }

    pub fn expect_stack(&self) -> Result<(usize, usize), String> {
        if let (TokenKind::STACK(_, i, j), _) = self.token[self.idx] {
            Ok((i, j))
        } else {
            let (kind, line) = self.token[self.idx].clone();
            Err(format!("{}: expect TokenKind::STACK(RegisterKind, usize, usize). but got: {:?}", line, kind))
        }
    }

    pub fn expect_integer(&self) -> Result<i32, String> {
        if let (TokenKind::INTEGER(i), _) = self.token[self.idx] {
            Ok(i)
        } else {
            let (kind, line) = self.token[self.idx].clone();
            Err(format!("{}: expect TokenKind::INTEGER(i32). but got: {:?}", line, kind))
        }
    }

    pub fn expect_eol(&self) -> Result<(), String> {
        if let (TokenKind::EOL, _) = self.token[self.idx] {
            Ok(())
        } else {
            let (kind, line) = self.token[self.idx].clone();
            Err(format!("{}: expect TokenKind::EOL. but got: {:?}", line, kind))
        }
    }
}

