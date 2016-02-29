

use std::mem;


/// The condition field of an ARM instruction.
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum ArmCondition {
    EQ = 0b0000, // Z set. EQual.
    NE = 0b0001, // Z clear. Not Equal.
    HS = 0b0010, // C set. Unsigned Higher or Same.
    LO = 0b0011, // C clear. Unsigned LOwer.
    MI = 0b0100, // N set. MInus, i.e. negative.
    PL = 0b0101, // N clear. PLus, i.e. positive or zero.
    VS = 0b0110, // V Set. Overflow.
    VC = 0b0111, // V Clear. No Overflow.
    HI = 0b1000, // C set and Z clear. Unsigned HIgher.
    LS = 0b1001, // C clear or Z set. Unsigned Lower or Same.
    GE = 0b1010, // N equals V. Greater than or Equal to.
    LT = 0b1011, // N distinct from V. Less Than.
    GT = 0b1100, // Z clear and N equals V. Greater Than.
    LE = 0b1101, // Z set or N distinct from V.  Less than or Equal to.
    AL = 0b1110, // ALways execute this instruction, i.e. no condition.
    NV = 0b1111, // Reserved.
}


// TODO
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ArmOpcode {
    Invalid,
    
    // Branches.
    B,  // PC += offs
    BL, // LR = PC-4, PC += offs
    BX, // PC +=
    
    // Arithmetic.
    ADD, // D = Op1 + Op2
    ADC, // D = Op1 + Op2 + carry
    SUB, // D = Op1 - Op2
    SBC, // D = Op1 - Op2 + carry - 1
    RSB, // D = Op2 - Op1
    RSC, // D = Op2 - Op1 + carry - 1
    
    // Comparisons.
    CMP, // Op1 - Op2, only flags set
    CMN, // Op1 + Op2, only flags set
    TST, // Op1 & Op2, only flags set
    TEQ, // Op1 ^ Op2, only flags set
    
    // Logical Operations.
    AND, // D = Op1 & Op2
    EOR, // D = Op1 ^ Op2
    ORR, // D = Op1 | Op2
    BIC, // D = Op1 & !Op2, i.e. bit clear
    
    // Data Movement.
    MOV, // D = Op2
    MVN, // D = !Op2
    
    // Multiplication.
    MUL,  // Rd = Rm * Rs
    MLA,  // Rd = (Rm * Rs) + Rn
    MULL, // RdHi_RdLo = Rm * Rs
    MLAL, // RdHi_RdLo = (Rm * Rs) + RdHi_RdLo
    
    // Load/Store Instructions.
    LDRW, // Load word.
    STRW, // Store word.
    LDRH, // Load signed/unsigned halfword.
    STRH, // Store signed/unsigned halfword.
    LDRB, // Load signed/unsigned byte.
    STRB, // Store signed/unsigned byte.
    
    // Block data transfer.
    LDM,  // Load multiple words.
    STM,  // Store multiple words.
    
    // PSR transfer.
    MRS,
    MSR,
    
    // Swap.
    SWPW,
    SWPB,
    
    // Software Interrupt
    SWI,
    
    // Co-Processor stuff.
    CDP(u8), // Co-processor Data oPerations with opcode (u8).
}

impl ArmOpcode {
    /// Decodes an ARM data processing opcode from a 4-bit number.
    ///
    /// # Params
    /// - `b`: A 4-bit opcode. Upper half will be ignored.
    ///
    /// # Returns
    /// A data processing opcode.
    pub fn data_processing_from_bits(b: u8) -> ArmOpcode {
        match b & 0b1111 {
            0b0000 => ArmOpcode::AND,
            0b0001 => ArmOpcode::EOR,
            0b0010 => ArmOpcode::SUB,
            0b0011 => ArmOpcode::RSB,
            0b0100 => ArmOpcode::ADD,
            0b0101 => ArmOpcode::ADC,
            0b0110 => ArmOpcode::SBC,
            0b0111 => ArmOpcode::RSC,
            0b1000 => ArmOpcode::TST,
            0b1001 => ArmOpcode::TEQ,
            0b1010 => ArmOpcode::CMP,
            0b1011 => ArmOpcode::CMN,
            0b1100 => ArmOpcode::ORR,
            0b1101 => ArmOpcode::MOV,
            0b1110 => ArmOpcode::BIC,
            0b1111 => ArmOpcode::MVN,
            _ => unreachable!(),
        }
    }
}


/*
    Instruction Flags:
        .... ....  .... ....  .... ....  .... ....
        COND                                  RegM | BX #map: RegN => RegM
        COND    F  imm_ imm_  imm_ imm_  imm_ imm_ | B/BL
        COND   Ix  xxxS RegN  RegD shft  shft shft | Data Processing Op4(xxxx)...
               0                   _imm  _op0 RegM | ... RegM = RegM SHIFT(op) _imm_
               0                   RegS  0op1 RegM | ... RegM = RegM SHIFT(op) RegS
               1                   xxxx  imm_ imm_ | ... Immediate = imm_imm_ ROR 2*xxxx
        COND        P         RegD                 | MRS
        COND        P                         RegM | MSR by RegM
        COND   I    P              shft  shft shft | MSR by Immediate...
               0                   _imm  _op0 RegM | ... RegM = RegM SHIFT(op) _imm_
               0                   RegS  0op1 RegM | ... RegM = RegM SHIFT(op) RegS
               1                   xxxx  imm_ imm_ | ... Immediate = imm_imm_ ROR 2*xxxx
        COND         AS RegN  RegD RegS       RegM | MUL/MLA #map: RegN <=> RegD
        COND        UAS RegN  RegD RegS       RegM | MULL/MLAL #map: RdHi => RegN, RdLo => RegD
        COND   I+  -BWL RegN  RegD offs  offs offs | LDR/STR
        COND    +  - WL RegN  RegD        xx  RegM | LDRH/STRH/LDRSB/LDRSH depending on Op(xx)
        COND    +  - WL RegN  RegD imm_   xx  imm_ | LDRH/STRH/LDRSB/LDRSH depending on Op(xx) with Offset=imm_imm_
        COND    +  -RWL RegN  regs regs  regs regs | LDM/STM with register list regsregsregsregs
        COND        B   RegN  RegD            RegM | SWP
        COND       imm_ imm_  imm_ imm_  imm_ imm_ | SWI with comment
        COND       CPOP RegN  RegD CPID  xxx  RegM | CDP with CoCPU Op4(CPOP) and CP Info xxx
        COND       yyyL CprN  RegD CPID  xxx  CprM | MRC/MCR with CoCPU Op3(yyy) and CP Info xxx
        COND 110+  -NWL RegN  CprD CPID  imm_ imm_ | LDC/STC with unsigned Immediate
        COND    ?  ???? ????  ???? ????  ???  ???? | Unknown Instruction
        
    Full Instructions:
        .... ....  .... ....  .... ....  .... ....
        COND 0001  0010 1111  1111 1111  0001 RegM | BX #map: RegN => RegM
        COND 101F  imm_ imm_  imm_ imm_  imm_ imm_ | B/BL with signed offset
        COND 00Ix  xxxS RegN  RegD shft  shft shft | Data Processing Op(xxxx)
        COND 0001  0P00 1111  RegD 0000  0000 0000 | MRS
        COND 0001  0P10 1001  1111 0000  0000 RegM | MSR by RegM
        COND 00I1  0P10 1000  1111 shft  shft shft | MSR by imm
        COND 0000  00AS RegN  RegD RegS  1001 RegM | MUL/MLA #map: RegN <=> RegD
        COND 0000  1UAS RegN  RegD RegS  1001 RegM | MULL/MLAL #map: RdHi => RegN, RdLo => RegD
        COND 01I+  -BWL RegN  RegD offs  offs offs | LDR/STR
        COND 000+  -0WL RegN  RegD 0000  1xx1 RegM | LDRH/STRH/LDRSB/LDRSH depending on Op(xx)
        COND 000+  -1WL RegN  RegD imm_  1xx1 imm_ | LDRH/STRH/LDRSB/LDRSH depending on Op(xx) with Offset=imm_imm_
        COND 100+  -RWL RegN  regs regs  regs regs | LDM/STM with register list regsregsregsregs
        COND 0001  0B00 RegN  RegD 0000  1001 RegM | SWP
        COND 1111  imm_ imm_  imm_ imm_  imm_ imm_ | SWI with comment
        COND 1110  CPOP CprN  CprD CPID  xxx0 CprM | CDP with CoCPU Op4 CPOP and CP Info xxx
        COND 1110  yyyL CprN  RegD CPID  xxx1 CprM | MRC/MCR with CoCPU Op3 yyy and CP Info xxx
        COND 110+  -NWL RegN  CprD CPID  imm_ imm_ | LDC/STC with unsigned Immediate
        COND 011?  ???? ????  ???? ????  ???1 ???? | Unknown Instruction
    
    Bit Flags:
        I: 1=shftIsRegister,  0=shftIsImmediate
        F: 1=BranchWithLink,  0=BranchWithoutLink
        +: 1=PreIndexing,     0=PostIndexing
        -: 1=AddOffset,       0=SubtractOffset
        P: 1=SPSR,            0=CPSR
        U: 1=Signed,          0=Unsigned
        B: 1=TransferByte,    0=TransferWord
        R: 1=ForceUserMode,   0=NoForceUserMode
        N: 1=TransferAllRegs, 0=TransferSingleReg
        A: 1=Accumulate,      0=DoNotAccumulate
        W: 1=AutoIncrement,   0=NoWriteBack
        S: 1=SetFlags,        0=DoNotSetFlags
        L: 1=Load,            0=Store
        
    Shift format:
        I=0: shft shft shft
             _imm _op0 RegM // RegM = RegM SHIFT(op) _imm_
             RegS 0op1 RegM // RegM = RegM SHIFT(op) RegS
             
        I=1: shft shft shft
             xxxx imm_ imm_ // Immediate = imm_imm_ ROR 2*xxxx
    
    Offset format:
        I=0: offs offs offs
             imm_ imm_ imm_ // Immediate unsigned offset.
        
        I=1: offs offs offs
             _imm _op0 RegM // RegM = RegM SHIFT(op) _imm_
             RegS 0op1 RegM // RegM = RegM SHIFT(op) RegS
*/

// TODO
#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(non_snake_case)]
pub struct ArmInstruction {
    raw: i32,
}

impl ArmInstruction {
    /// Get the condition field of the ARM instruction.
    pub fn condition(self) -> ArmCondition {
        let c = ((self.raw >> 28) & 0b1111) as u8;
        unsafe { mem::transmute(c) }
    }
    
    /// Get the index of register `Rn`.
    #[allow(non_snake_case)]
    pub fn Rn(self) -> usize {
        ((self.raw >> 16) & 0b1111) as usize
    }
    
    /// Get the index of register `Rd`.
    #[allow(non_snake_case)]
    pub fn Rd(self) -> usize {
        ((self.raw >> 12) & 0b1111) as usize
    }
    
    /// Get the index of register `Rs`.
    #[allow(non_snake_case)]
    pub fn Rs(self) -> usize {
        ((self.raw >> 8) & 0b1111) as usize
    }
    
    /// Get the index of register `Rm`.
    #[allow(non_snake_case)]
    pub fn Rm(self) -> usize {
        ((self.raw >> 0) & 0b1111) as usize
    }
    
    /// Calculates a shifted operand without carry flag.
    ///
    /// In case the operand is a rotated immediate, this
    /// immediate value is returned. Otherwise, the given
    /// registers are used to calculate the operand.
    ///
    /// # Params
    /// - `regs`: The CPU's GPRs.
    /// - `carry`: The current status of the carry flag.
    ///
    /// # Returns
    /// A 32-bit operand.
    pub fn shifted_operand(self, regs: &[i32], carry: bool) -> i32 {
        if self.is_shift_field_register() {
            self.calculate_shifted_register(regs, carry)
        }
        else { self.rotated_immediate() }
    }
    
    /// Calculates a shifted operand with carry flag.
    ///
    /// In case the operand is a rotated immediate, this
    /// immediate value is returned. Otherwise, the given
    /// registers are used to calculate the operand.
    ///
    /// # Params
    /// - `regs`: The CPU's GPRs.
    /// - `carry`: The current status of the carry flag.
    ///
    /// # Returns
    /// - `.0`: A 32-bit operand.
    /// - `.1`: The new status of the carry flag.
    pub fn shifted_operand_with_carry(self, regs: &[i32], carry: bool) -> (i32, bool) {
        if self.is_shift_field_register() {
            self.calculate_shifted_register_with_carry(regs, carry)
        }
        else { (self.rotated_immediate(), false) }
    }
    
    /// Get the 24-bit sign-extended branch offset.
    pub fn branch_offset(self) -> i32 {
        ((self.raw << 8) as i32) >> 8
    }
    
    /// Get the 24-bit comment field of an `SWI` instruction.
    pub fn comment(self) -> i32 {
        (self.raw & 0x00FFFFFF) as i32
    }
    
    /// Determines whether a shift field is to be decoded as
    /// rotated immediate value or as a shifted register value.
    ///
    /// # Returns
    /// - `true`: Shift field is a register shift.
    /// - `false`: Shift field is a rotated immediate.
    pub fn is_shift_field_register(self) -> bool {
        (self.raw & (1 << 25)) != 0
    }
    
    /// Decodes a rotated immediate value.
    ///
    /// # Returns
    /// An immediate 32-bit value consisting of a single
    /// rotated byte.
    pub fn rotated_immediate(self) -> i32 {
        let bits = (2 * ((self.raw >> 8) & 0b1111)) as u32;
        (self.raw & 0xFF).rotate_right(bits) as i32
    }
    
    /// Determines whether an offset field is to be decoded
    /// as shifted registers or as an immediate value.
    ///
    /// # Returns
    /// - `true`: The offset is an immediate non-sign-extended value.
    /// - `false`: The offset is a shifted register value.
    #[inline(always)]
    pub fn is_offset_field_immediate(self) -> bool {
        self.is_shift_field_register()
    }
    
    /// Checks whether this is a `B` or `BL` instruction.
    ///
    /// # Returns:
    /// - `true`: `BL`
    /// - `false`: `B`
    pub fn is_branch_with_link(self) -> bool {
        (self.raw & (1 << 24)) != 0
    }
    
    /// Checks whether an offset register should be
    /// pre-indexed or post-indexed.
    ///
    /// # Returns:
    /// - `true`: pre-indexed
    /// - `false`: post-indexed
    pub fn is_pre_indexed(self) -> bool {
        (self.raw & (1 << 24)) != 0
    }
    
    /// Checks whether a given offset should be added
    /// or subtracted from a base address.
    ///
    /// # Returns
    /// - `true`: Add the given offset to the base address.
    /// - `false`: Subtract the given offset from the base address.
    pub fn is_offset_added(self) -> bool {
        (self.raw & (1 << 23)) != 0
    }
    
    /// Checks whether the given instruction accesses CPSR
    /// or the current SPSR.
    ///
    /// # Returns
    /// - `true`: Accessing the current SPSR.
    /// - `false`: Accessing CPSR.
    pub fn is_accessing_spsr(self) -> bool {
        (self.raw & (1 << 22)) != 0
    }
    
    /// Checks whether the given long instruction should
    /// act as a signed or unsigned operation.
    ///
    /// # Returns
    /// - `true`: The operation is signed.
    /// - `false`: The operation is unsigned.
    pub fn is_signed(self) -> bool {
        (self.raw & (1 << 22)) != 0
    }
    
    /// Checks whether a data transfer instruction should
    /// transfer bytes or words.
    ///
    /// # Returns
    /// - `true`: Transfer bytes.
    /// - `false`: Transfering words.
    pub fn is_transfering_bytes(self) -> bool {
        (self.raw & (1 << 22)) != 0
    }
    
    /// Checks whether register block transfer should be
    /// done in user mode.
    ///
    /// # Returns
    /// - `true`: Enforce user mode for privileged code.
    /// - `false`: Execute in current mode.
    pub fn is_enforcing_user_mode(self) -> bool {
        (self.raw & (1 << 22)) != 0
    }
    
    /// Checks whether a single register or a block of
    /// registers should be transfered to or from a
    /// co-processor.
    ///
    /// # Returns
    /// - `true`: Transfer a block of registers.
    /// - `false`: Transfer a single register.
    pub fn is_register_block_transfer(self) -> bool {
        (self.raw & (1 << 22)) != 0
    }
    
    /// Checks whether a multiply instruction should
    /// accumulate or not.
    ///
    /// # Returns
    /// - `true`: Accumulate.
    /// - `false`: Don't accumulate.
    pub fn is_accumulating(self) -> bool {
        (self.raw & (1 << 21)) != 0
    }
    
    /// Checks whether the current instruction writes
    /// a calculated address back to the base register.
    pub fn is_auto_incrementing(self) -> bool {
        (self.raw & (1 << 21)) != 0
    }
    
    /// Checks whether the given instruction updates the
    /// ZNCV status flags of CPSR.
    ///
    /// # Returns
    /// - `true`: Updates CPSR.
    /// - `false`: Does not modify CPSR.
    pub fn is_setting_flags(self) -> bool {
        (self.raw & (1 << 20)) != 0
    }
    
    /// Checks whether the given instruction is a
    /// load or store instruction.
    ///
    /// # Returns
    /// - `true`: Load instruction.
    /// - `false`: Store instruction.
    pub fn is_load(self) -> bool {
        (self.raw & (1 << 20)) != 0
    }
    
    /// Calculates a shifted register operand without
    /// calculating the carry flag.
    ///
    /// # Params
    /// - `regs`: A reference to the CPU's general purpose registers.
    ///
    /// # Returns
    /// A ready-to-use operand.
    pub fn calculate_shifted_register(self, regs: &[i32], carry: bool) -> i32 {
        // 0000 0000 RegM // ret = RegM
        // _imm _op0 RegM // ret = RegM SHIFT(op) _imm_
        // RegS 0op1 RegM // ret = RegM SHIFT(op) RegS
        // 0000 0110 RegM // ret = RegM RRX 1
        let a = regs[self.Rm()];
        if (self.raw & 0x0FF0) == 0 { return a; }
        
        // Decode RRX?
        if (self.raw & 0x0FF0) == 0x60 {
            let bit31: i32 = if carry { 0x80000000_u32 as i32 } else { 0 };
            return bit31 | (((a as u32) >> 1) as i32);
        }
        
        let b: u32 = if (self.raw & (1 << 4)) == 0 {
            ((self.raw >> 7) & 0b1_1111) as u32
        } else {
            match self.decode_Rs_shift(a, regs) {
                Ok(x) => x,
                Err(y) => { return y; },
            }
        };
        
        match (self.raw >> 5) & 0b11 {
            0 => a.wrapping_shl(b),
            1 => (a as u32).wrapping_shr(b) as i32,
            2 => a.wrapping_shr(b),
            3 => a.rotate_right(b % 32),
            _ => unreachable!(),
        }
    }
    
    /// Calculates a shifted register operand and
    /// calculates the resulting carry flag.
    ///
    /// # Params
    /// - `regs`: A reference to the CPU's general purpose registers.
    /// - `carry`: The current state of the carry flag.
    ///
    /// # Returns
    /// - `.0`: A ready-to-use operand.
    /// - `.1`: `true` if carry, otherwise `false`.
    pub fn calculate_shifted_register_with_carry(self, regs: &[i32], carry: bool) -> (i32, bool) {
        // 0000 0000 RegM // ret = RegM
        // _imm _op0 RegM // ret = RegM SHIFT(op) _imm_
        // RegS 0op1 RegM // ret = RegM SHIFT(op) RegS
        // 0000 0110 RegM // ret = RegM RRX 1
        let a = regs[self.Rm()];
        if (self.raw & 0x0FF0) == 0 { return (a, false); }
        
        // Decode RRX?
        if (self.raw & 0x0FF0) == 0x60 {
            let bit31: i32 = if carry { 0x80000000_u32 as i32 } else { 0 };
            return (bit31 | (((a as u32) >> 1) as i32), 0 != (a & 0b1));
        }
        
        // A shift by Rs==0 just returns a without `a` new carry flag.
        let b: u32 = if (self.raw & (1 << 4)) == 0 {
            ((self.raw >> 7) & 0b1_1111) as u32
        } else {
            match self.decode_Rs_shift_with_carry(a, regs, carry) {
                Ok(x) => x,
                Err(y) => { return y; },
            }
        };
        
        let carry_right = 0 != (a.wrapping_shr(b - 1) & 0b1);
        
        match (self.raw >> 5) & 0b11 {
            0 => (a.wrapping_shl(b), 0 != (a.wrapping_shr(32 - b) & 0b1)),
            1 => ((a as u32).wrapping_shr(b) as i32, carry_right),
            2 => (a.wrapping_shr(b), carry_right),
            3 => (a.rotate_right(b % 32), carry_right),
            _ => unreachable!(),
        }
    }
    
    
    #[allow(non_snake_case)]
    fn decode_Rs_shift(self, a: i32, regs: &[i32]) -> Result<u32, i32> {
        let r = (regs[self.Rs()] & 0xFF) as u32;
        if r == 0 { return Err(a); }
        
        if r >= 32 {
            Err(match (self.raw >> 5) & 0b11 {
                0 => 0,
                1 => 0,
                2 => a >> 31,
                3 => if r == 32 { a } else { a.rotate_right(r % 32) },
                _ => unreachable!(),
            })
        }
        else { Ok(r) }
    }
    
    #[allow(non_snake_case)]
    fn decode_Rs_shift_with_carry(self, a: i32, regs: &[i32], carry: bool) -> Result<u32, (i32, bool)> {
        let r = (regs[self.Rs()] & 0xFF) as u32;
        if r == 0 { return Err((a, carry)); }
        
        if r >= 32 {
            let bit31 = 0 != (a & (0x80000000_u32 as i32));
            Err(match (self.raw >> 5) & 0b11 {
                0 => if r == 32 { (0, 0 != (a & 0b1)) } else { (0, false) },
                1 => if r == 32 { (0, bit31) } else { (0, false) },
                2 => (a >> 31, bit31),
                3 => if r == 32 { (a, bit31) } else {
                    let r = r % 32;
                    (a.rotate_right(r), 0 != (a.wrapping_shr(r - 1) & 0b1))
                },
                _ => unreachable!(),
            })
        }
        else { Ok(r) }
    }
}
