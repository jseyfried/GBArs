

use std::mem;
use std::fmt;

use super::super::error::GbaError;


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

impl fmt::Display for ArmCondition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ArmCondition::EQ => write!(f, "eq"),
            ArmCondition::NE => write!(f, "ne"),
            ArmCondition::HS => write!(f, "hs"),
            ArmCondition::LO => write!(f, "lo"),
            ArmCondition::MI => write!(f, "mi"),
            ArmCondition::PL => write!(f, "pl"),
            ArmCondition::VS => write!(f, "vs"),
            ArmCondition::VC => write!(f, "vc"),
            ArmCondition::HI => write!(f, "hi"),
            ArmCondition::LS => write!(f, "ls"),
            ArmCondition::GE => write!(f, "ge"),
            ArmCondition::LT => write!(f, "lt"),
            ArmCondition::GT => write!(f, "gt"),
            ArmCondition::LE => write!(f, "le"),
            ArmCondition::AL => write!(f,   ""), // No special name here!
            ArmCondition::NV => write!(f, "nv"),
        }
    }
}


/// A decoded ARM opcode.
#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum ArmOpcode {
    BX,
    B_BL,
    MUL_MLA,
    MULL_MLAL,
    DataProcessing,
    MRS,
    MSR_Reg,
    MSR_Immediate,
    LDR_STR,
    LDRH_STRH_Reg,
    LDRH_STRH_Immediate,
    LDM_STM,
    SWP,
    SWI,
    CDP,
    MRC_MCR,
    LDC_STC,
    Unknown,
}


/// A data processing opcode.
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum ArmDPOP {
    AND = 0b0000,
    EOR = 0b0001,
    SUB = 0b0010,
    RSB = 0b0011,
    ADD = 0b0100,
    ADC = 0b0101,
    SBC = 0b0110,
    RSC = 0b0111,
    TST = 0b1000,
    TEQ = 0b1001,
    CMP = 0b1010,
    CMN = 0b1011,
    ORR = 0b1100,
    MOV = 0b1101,
    BIC = 0b1110,
    MVN = 0b1111,
}

impl ArmDPOP {
    /// Checks whether this instruction does not
    /// write any results to a destination register.
    pub fn is_test(self) -> bool {
        match self {
            ArmDPOP::TST | ArmDPOP::TEQ | ArmDPOP::CMP | ArmDPOP::CMN => true,
            _ => false,
        }
    }
    
    /// Checks whether this instruction is a
    /// move instruction.
    pub fn is_move(self) -> bool {
        match self {
            ArmDPOP::MOV | ArmDPOP::MVN => true,
            _ => false,
        }
    }
}

impl fmt::Display for ArmDPOP {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ArmDPOP::AND => write!(f, "and"),
            ArmDPOP::EOR => write!(f, "eor"),
            ArmDPOP::SUB => write!(f, "sub"),
            ArmDPOP::RSB => write!(f, "rsb"),
            ArmDPOP::ADD => write!(f, "add"),
            ArmDPOP::ADC => write!(f, "adc"),
            ArmDPOP::SBC => write!(f, "sbc"),
            ArmDPOP::RSC => write!(f, "rsc"),
            ArmDPOP::TST => write!(f, "tst"),
            ArmDPOP::TEQ => write!(f, "teq"),
            ArmDPOP::CMP => write!(f, "cmp"),
            ArmDPOP::CMN => write!(f, "cmn"),
            ArmDPOP::ORR => write!(f, "orr"),
            ArmDPOP::MOV => write!(f, "mov"),
            ArmDPOP::BIC => write!(f, "bic"),
            ArmDPOP::MVN => write!(f, "mvn"),
        }
    }
}


/*
    Instruction Flags:
        .... ....  .... ....  .... ....  .... ....
        COND                                  RegM | BX #map: RegN => RegM
        COND    F  imm_ imm_  imm_ imm_  imm_ imm_ | B/BL
        COND         AS RegN  RegD RegS       RegM | MUL/MLA #map: RegN <=> RegD
        COND        UAS RegN  RegD RegS       RegM | MULL/MLAL #map: RdHi => RegN, RdLo => RegD
        COND        P         RegD                 | MRS
        COND        P                         RegM | MSR by RegM
        COND   I    P              shft  shft shft | MSR by Immediate...
               0                   _imm  _op0 RegM | ... RegM = RegM SHIFT(op) _imm_
               0                   RegS  0op1 RegM | ... RegM = RegM SHIFT(op) RegS
               1                   xxxx  imm_ imm_ | ... Immediate = imm_imm_ ROR 2*xxxx
        COND   Ix  xxxS RegN  RegD shft  shft shft | Data Processing Op4(xxxx)...
               0                   _imm  _op0 RegM | ... RegM = RegM SHIFT(op) _imm_
               0                   RegS  0op1 RegM | ... RegM = RegM SHIFT(op) RegS
               1                   xxxx  imm_ imm_ | ... Immediate = imm_imm_ ROR 2*xxxx
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
        COND 0000  00AS RegN  RegD RegS  1001 RegM | MUL/MLA #map: RegN <=> RegD
        COND 0000  1UAS RegN  RegD RegS  1001 RegM | MULL/MLAL #map: RdHi => RegN, RdLo => RegD
        COND 0001  0P00 1111  RegD 0000  0000 0000 | MRS
        COND 0001  0P10 1001  1111 0000  0000 RegM | MSR by RegM
        COND 00I1  0P10 1000  1111 shft  shft shft | MSR by imm
        COND 00Ix  xxxS RegN  RegD shft  shft shft | Data Processing Op(xxxx)
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

/// A decoded ARM instruction providing lots
/// of utility and decoding functions to ease
/// ARM instruction emulation.
#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(non_snake_case)]
pub struct ArmInstruction {
    raw: i32,
    op: ArmOpcode,
}

impl ArmInstruction {
    /// A raw 32-bit pseudo NOP instruction.
    pub const NOP_RAW: i32 = 0b0000_00_0_1101_0_0000_0000_00000000_0000_u32 as i32;
    //                         COND DP I MOV  S  Rn   Rd   LSL(0)   Rm
    
    /// Creates a pseudo NOP instruction.
    ///
    /// # Returns
    /// The generated instruction moves `R0` into `R0` without any shifting
    /// and only executes on the condition `EQ`.
    pub fn nop() -> ArmInstruction {
        ArmInstruction { raw: ArmInstruction::NOP_RAW, op: ArmOpcode::DataProcessing }
    }
    
    /// Decodes a raw 32-bit integer as an ARM instruction.
    ///
    /// Unknown instructions do not result in a decoding
    /// error, as they cause specific exceptions in an ARM
    /// processor.
    ///
    /// # Params
    /// - `raw`: The raw 32-bit integer.
    ///
    /// # Returns
    /// - `Ok`: A successfully decoded ARM instruction.
    /// - `Err`: In case any unspecified instruction has been decoded.
    pub fn decode(raw: i32) -> Result<ArmInstruction, GbaError> {
        // Decode the opcode to something easier to match and read.
        let op: ArmOpcode =
             if (raw & 0x0FFFFFF0) == 0x012FFF10 { ArmOpcode::BX }
        else if (raw & 0x0E000000) == 0x0A000000 { ArmOpcode::B_BL }
        else if (raw & 0x0FC000F0) == 0x00000090 { ArmOpcode::MUL_MLA }
        else if (raw & 0x0F8000F0) == 0x00800090 { ArmOpcode::MULL_MLAL }
        else if (raw & 0x0FBF0FFF) == 0x010F0000 { ArmOpcode::MRS }
        else if (raw & 0x0FBFFFF0) == 0x0129F000 { ArmOpcode::MSR_Reg }
        else if (raw & 0x0DBFF000) == 0x0128F000 { ArmOpcode::MSR_Immediate }
        else if (raw & 0x0C000000) == 0x00000000 { ArmOpcode::DataProcessing }
        else if (raw & 0x0C000000) == 0x04000000 { ArmOpcode::LDR_STR }
        else if (raw & 0x0E400F90) == 0x00000090 { ArmOpcode::LDRH_STRH_Reg }
        else if (raw & 0x0E400090) == 0x00400090 { ArmOpcode::LDRH_STRH_Immediate }
        else if (raw & 0x0E000000) == 0x08000000 { ArmOpcode::LDM_STM }
        else if (raw & 0x0FB00FF0) == 0x01000090 { ArmOpcode::SWP }
        else if (raw & 0x0F000000) == 0x0F000000 { ArmOpcode::SWI }
        else if (raw & 0x0F000010) == 0x0E000000 { ArmOpcode::CDP }
        else if (raw & 0x0F000010) == 0x0E000010 { ArmOpcode::MRC_MCR }
        else if (raw & 0x0E000000) == 0x0C000000 { ArmOpcode::LDC_STC }
        else if (raw & 0x0E000010) == 0x06000010 { ArmOpcode::Unknown }
        else {
            return Err(GbaError::InvalidArmInstruction(raw as u32));
        };
        
        // Done decoding!
        Ok(ArmInstruction { raw: raw, op: op })
    }
    
    /// Get the condition field of the ARM instruction.
    pub fn condition(&self) -> ArmCondition {
        let c = ((self.raw >> 28) & 0b1111) as u8;
        unsafe { mem::transmute(c) }
    }
    
    /// Get the data processing opcode field of the ARM instruction.
    pub fn dpop(&self) -> ArmDPOP {
        let o = ((self.raw >> 21) & 0b1111) as u8;
        unsafe { mem::transmute(o) }
    }
    
    /// Get the index of register `Rn`.
    #[allow(non_snake_case)]
    pub fn Rn(&self) -> usize {
        ((self.raw >> 16) & 0b1111) as usize
    }
    
    /// Get the index of register `Rd`.
    #[allow(non_snake_case)]
    pub fn Rd(&self) -> usize {
        ((self.raw >> 12) & 0b1111) as usize
    }
    
    /// Get the index of register `Rs`.
    #[allow(non_snake_case)]
    pub fn Rs(&self) -> usize {
        ((self.raw >> 8) & 0b1111) as usize
    }
    
    /// Get the index of register `Rm`.
    #[allow(non_snake_case)]
    pub fn Rm(&self) -> usize {
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
    pub fn shifted_operand(&self, regs: &[i32], carry: bool) -> i32 {
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
    pub fn shifted_operand_with_carry(&self, regs: &[i32], carry: bool) -> (i32, bool) {
        if self.is_shift_field_register() {
            self.calculate_shifted_register_with_carry(regs, carry)
        }
        else { (self.rotated_immediate(), false) }
    }
    
    /// Calculates a shifted offset for a base address.
    ///
    /// In case the offset is a rotated immediate, this
    /// immediate value is returned. Otherwise, the given
    /// registers are used to calculate the offset.
    ///
    /// Additionally, this function checks the additive/
    /// subtractive offset flag. If the offset is to be
    /// subtracted, `-offset` will be returned, otherwise
    /// `+offset` will be returned. Thus, the returned
    /// offset is signed and should be added to any given
    /// base address.
    ///
    /// # Params
    /// - `regs`: The CPU's GPRs.
    /// - `carry`: The current status of the carry flag.
    ///
    /// # Returns
    /// A signed offset that should be added to any given
    /// base address.
    pub fn shifted_offset(&self, regs: &[i32], carry: bool) -> i32 {
        let offs = if self.is_offset_field_immediate() {
            self.raw & 0x0FFF
        } else {
            self.calculate_shifted_register(regs, carry)
        };
        
        if self.is_offset_added() { offs } else { -offs }
    }
    
    /// Gets a zero-extended 8-bit immediate to be used with LDC/STC.
    pub fn offset8(&self) -> i32 {
        (self.raw & 0xFF) as i32
    }
    
    /// Gets an 8-bit offset to be used with LDRH/STRH/LDRSB/LDRSH.
    ///
    /// The offset has been split into two nibbles. This function
    /// re-combines them and also checks the add/sub flag. If the
    /// flag is set to subtract, then `-offset` will be returned,
    /// otherwise `offset` will be returned.
    pub fn split_offset8(&self) -> i32 {
        let off = ((self.raw >> 4) & 0xF0) | (self.raw & 0x0F);
        if self.is_offset_added() { off } else { -off }
    }
    
    /// Get the 24-bit sign-extended branch offset.
    pub fn branch_offset(&self) -> i32 {
        ((self.raw << 8) as i32) >> 6
    }
    
    /// Get the 24-bit comment field of an `SWI` instruction.
    pub fn comment(&self) -> i32 {
        (self.raw & 0x00FFFFFF) as i32
    }
    
    /// Get a 16-bit bitmap, where bit N corresponds to GPR N.
    pub fn register_map(&self) -> u16 {
        (self.raw & 0xFFFF) as u16
    }
    
    /// Determines whether a shift field is to be decoded as
    /// rotated immediate value or as a shifted register value.
    ///
    /// # Returns
    /// - `true`: Shift field is a register shift.
    /// - `false`: Shift field is a rotated immediate.
    pub fn is_shift_field_register(&self) -> bool {
        (self.raw & (1 << 25)) == 0
    }
    
    /// Decodes a rotated immediate value.
    ///
    /// # Returns
    /// An immediate 32-bit value consisting of a single
    /// rotated byte.
    pub fn rotated_immediate(&self) -> i32 {
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
    pub fn is_offset_field_immediate(&self) -> bool {
        self.is_shift_field_register()
    }
    
    /// Checks whether this is a `B` or `BL` instruction.
    ///
    /// # Returns:
    /// - `true`: `BL`
    /// - `false`: `B`
    pub fn is_branch_with_link(&self) -> bool {
        (self.raw & (1 << 24)) != 0
    }
    
    /// Checks whether an offset register should be
    /// pre-indexed or post-indexed.
    ///
    /// # Returns:
    /// - `true`: pre-indexed
    /// - `false`: post-indexed
    pub fn is_pre_indexed(&self) -> bool {
        (self.raw & (1 << 24)) != 0
    }
    
    /// Checks whether a given offset should be added
    /// or subtracted from a base address.
    ///
    /// # Returns
    /// - `true`: Add the given offset to the base address.
    /// - `false`: Subtract the given offset from the base address.
    pub fn is_offset_added(&self) -> bool {
        (self.raw & (1 << 23)) != 0
    }
    
    /// Checks whether the given instruction accesses CPSR
    /// or the current SPSR.
    ///
    /// # Returns
    /// - `true`: Accessing the current SPSR.
    /// - `false`: Accessing CPSR.
    pub fn is_accessing_spsr(&self) -> bool {
        (self.raw & (1 << 22)) != 0
    }
    
    /// Checks whether the given long instruction should
    /// act as a signed or unsigned operation.
    ///
    /// # Returns
    /// - `true`: The operation is signed.
    /// - `false`: The operation is unsigned.
    pub fn is_signed(&self) -> bool {
        (self.raw & (1 << 22)) != 0
    }
    
    /// Checks whether a data transfer instruction should
    /// transfer bytes or words.
    ///
    /// # Returns
    /// - `true`: Transfer bytes.
    /// - `false`: Transfering words.
    pub fn is_transfering_bytes(&self) -> bool {
        (self.raw & (1 << 22)) != 0
    }
    
    /// Checks whether register block transfer should be
    /// done in user mode.
    ///
    /// # Returns
    /// - `true`: Enforce user mode for privileged code.
    /// - `false`: Execute in current mode.
    pub fn is_enforcing_user_mode(&self) -> bool {
        (self.raw & (1 << 22)) != 0
    }
    
    /// Checks whether a single register or a block of
    /// registers should be transfered to or from a
    /// co-processor.
    ///
    /// # Returns
    /// - `true`: Transfer a block of registers.
    /// - `false`: Transfer a single register.
    pub fn is_register_block_transfer(&self) -> bool {
        (self.raw & (1 << 22)) != 0
    }
    
    /// Checks whether a multiply instruction should
    /// accumulate or not.
    ///
    /// # Returns
    /// - `true`: Accumulate.
    /// - `false`: Don't accumulate.
    pub fn is_accumulating(&self) -> bool {
        (self.raw & (1 << 21)) != 0
    }
    
    /// Checks whether the current instruction writes
    /// a calculated address back to the base register.
    pub fn is_auto_incrementing(&self) -> bool {
        (self.raw & (1 << 21)) != 0
    }
    
    /// Checks whether the given instruction updates the
    /// ZNCV status flags of CPSR.
    ///
    /// # Returns
    /// - `true`: Updates CPSR.
    /// - `false`: Does not modify CPSR.
    pub fn is_setting_flags(&self) -> bool {
        (self.raw & (1 << 20)) != 0
    }
    
    /// Checks whether the given instruction is a
    /// load or store instruction.
    ///
    /// # Returns
    /// - `true`: Load instruction.
    /// - `false`: Store instruction.
    pub fn is_load(&self) -> bool {
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
    pub fn calculate_shifted_register(&self, regs: &[i32], carry: bool) -> i32 {
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
    pub fn calculate_shifted_register_with_carry(&self, regs: &[i32], carry: bool) -> (i32, bool) {
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
    fn decode_Rs_shift(&self, a: i32, regs: &[i32]) -> Result<u32, i32> {
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
    fn decode_Rs_shift_with_carry(&self, a: i32, regs: &[i32], carry: bool) -> Result<u32, (i32, bool)> {
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
    
    
    fn display_shift(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_shift_field_register() {
            try!(write!(f, "R{}", self.Rm()));
            self.display_shift_op(f)
        }
        else { write!(f, "#{}", self.rotated_immediate()) }
    }
    
    fn display_offset(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write the base register.
        try!(write!(f, "[R{}{}, ",
            self.Rn(),
            if self.is_pre_indexed() { "" } else { "]" }
        ));
        
        // Write the offset.
        if self.is_offset_field_immediate() {
            try!(write!(f, "#{}{}", if self.is_offset_added() { "" } else { "+" }, self.raw & 0x0FFF));
        } else {
            try!(write!(f, "{}R{}",
                if self.is_offset_added() { "+" } else { "-" }, self.Rm(),
            ));
            try!(self.display_shift_op(f));
        }
        
        // Add bracket if pre-indexed.
        write!(f, "{}{}",
            if self.is_pre_indexed() { "]" } else { "" },
            if self.is_auto_incrementing() { "!" } else { "" }
        )
    }
    
    fn display_shift_op(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Ignore LSL(0) and handle RRX.
        if (self.raw & 0x0FF0) == 0    { return Ok(()); }
        if (self.raw & 0x0FF0) == 0x60 { return write!(f, ", rrx"); }
        
        // First write the shift opcode.
        try!(match (self.raw >> 5) & 0b11 {
            0 => write!(f, ", lsl "),
            1 => write!(f, ", lsr "),
            2 => write!(f, ", asr "),
            3 => write!(f, ", ror "),
            _ => unreachable!(),
        });
        
        // Register or immediate?
        if (self.raw & (1 << 4)) == 0 { write!(f, "#{}", (self.raw >> 7) & 0b1_1111) }
        else                          { write!(f, "R{}", self.Rs()) }
    }
}



impl fmt::Display for ArmInstruction {
    /// Just a big mess of code generating a disassembly for
    /// the current ARM instruction.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{:#08X}\t", self.raw as u32));
        
        let cond = self.condition();
        
        match self.op {
            ArmOpcode::BX             => write!(f, "bx{}\tR{}", cond, self.Rm()),
            ArmOpcode::B_BL           => write!(f, "b{}{}\t{}", if self.is_branch_with_link() { "l" } else { "" }, cond, 8+self.branch_offset()),
            ArmOpcode::MRS            => write!(f, "mrs{}\tR{}, {}", cond, self.Rd(), if self.is_accessing_spsr() { "SPSR" } else { "CPSR" }),
            ArmOpcode::MSR_Reg        => write!(f, "msr{}\t{}, R{}", cond, if self.is_accessing_spsr() { "SPSR" } else { "CPSR" }, self.Rm()),
            ArmOpcode::LDR_STR        => {
                try!(write!(f, "{}{}{}{}\tR{}, ",
                    if self.is_load() { "ldr" } else { "str" }, cond,
                    if self.is_transfering_bytes() { "b" } else { "" },
                    if self.is_pre_indexed() { "" } else { "t" },
                    self.Rd()
                ));
                self.display_offset(f)
            },
            
            
            ArmOpcode::MUL_MLA        => {
                try!(write!(f, "{}{}{}\t, R{}, R{}, R{}",
                    if self.is_accumulating() { "mla" } else { "mul" },
                    cond, if self.is_setting_flags() { "s" } else { "" },
                    self.Rn(), self.Rm(), self.Rs(),
                ));
                if self.is_accumulating() {
                    write!(f, ", R{}", self.Rd())
                }
                else { Ok(()) }
            },
            ArmOpcode::MULL_MLAL      => write!(f, "{}{}{}{}\t, R{}, R{}, R{}, R{}",
                    if self.is_signed() { "s" } else { "u" },
                    if self.is_accumulating() { "mlal" } else { "mull" },
                    cond, if self.is_setting_flags() { "s" } else { "" },
                    self.Rd(), self.Rn(), self.Rm(), self.Rs(),
            ),
            ArmOpcode::MSR_Immediate  => {
                try!(write!(f, "msr{}\t{}, ", cond, if self.is_accessing_spsr() { "SPSR" } else { "CPSR" }));
                self.display_shift(f)
            },
            ArmOpcode::DataProcessing => {
                let op = self.dpop();
                try!(write!(f, "{}{}{}\t", &op, cond, if self.is_setting_flags() && !op.is_test() { "s" } else { "" }));
                if !op.is_test() { try!(write!(f, "R{}, ", self.Rd())); }
                if !op.is_move() { try!(write!(f, "R{}, ", self.Rn())); }
                self.display_shift(f)
            },
            
            _ => unimplemented!(),
        }
    }
}