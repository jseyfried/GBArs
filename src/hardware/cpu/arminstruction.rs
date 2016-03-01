// License below.
//! Implements utilities to decode, disassemble, and handle
//! 32-bit ARM state instructions.
//!
//! These tables show how ARM state instructions are encoded:
//!
//! ```text
//! Instruction Flags:
//!     .... ....  .... ....  .... ....  .... ....
//!     COND                                  RegM | BX #map: RegN => RegM
//!     COND    F  imm_ imm_  imm_ imm_  imm_ imm_ | B/BL
//!     COND         AS RegN  RegD RegS       RegM | MUL/MLA #map: RegN <=> RegD
//!     COND        UAS RegN  RegD RegS       RegM | MULL/MLAL #map: RdHi => RegN, RdLo => RegD
//!     COND        P         RegD                 | MRS
//!     COND        P                         RegM | MSR by RegM
//!     COND   I    P              shft  shft shft | MSR by Immediate...
//!            0                   _imm  _op0 RegM | ... RegM = RegM SHIFT(op) _imm_
//!            0                   RegS  0op1 RegM | ... RegM = RegM SHIFT(op) RegS
//!            1                   xxxx  imm_ imm_ | ... Immediate = imm_imm_ ROR 2*xxxx
//!     COND   Ix  xxxS RegN  RegD shft  shft shft | Data Processing Op4(xxxx)...
//!            0                   _imm  _op0 RegM | ... RegM = RegM SHIFT(op) _imm_
//!            0                   RegS  0op1 RegM | ... RegM = RegM SHIFT(op) RegS
//!            1                   xxxx  imm_ imm_ | ... Immediate = imm_imm_ ROR 2*xxxx
//!     COND   I+  -BWL RegN  RegD offs  offs offs | LDR/STR
//!     COND    +  - WL RegN  RegD        xx  RegM | LDRH/STRH/LDRSB/LDRSH depending on Op(xx)
//!     COND    +  - WL RegN  RegD imm_   xx  imm_ | LDRH/STRH/LDRSB/LDRSH depending on Op(xx) with Offset=imm_imm_
//!     COND    +  -RWL RegN  regs regs  regs regs | LDM/STM with register list regsregsregsregs
//!     COND        B   RegN  RegD            RegM | SWP
//!     COND       imm_ imm_  imm_ imm_  imm_ imm_ | SWI with comment
//!     COND       CPOP RegN  RegD CPID  xxx  RegM | CDP with CoCPU Op4(CPOP) and CP Info xxx
//!     COND       yyyL CprN  RegD CPID  xxx  CprM | MRC/MCR with CoCPU Op3(yyy) and CP Info xxx
//!     COND 110+  -NWL RegN  CprD CPID  imm_ imm_ | LDC/STC with unsigned Immediate
//!     COND    ?  ???? ????  ???? ????  ???  ???? | Unknown Instruction
//!     
//! Full Instructions:
//!     .... ....  .... ....  .... ....  .... ....
//!     COND 0001  0010 1111  1111 1111  0001 RegM | BX #map: RegN => RegM
//!     COND 101F  imm_ imm_  imm_ imm_  imm_ imm_ | B/BL with signed offset
//!     COND 0000  00AS RegN  RegD RegS  1001 RegM | MUL/MLA #map: RegN <=> RegD
//!     COND 0000  1UAS RegN  RegD RegS  1001 RegM | MULL/MLAL #map: RdHi => RegN, RdLo => RegD
//!     COND 0001  0P00 1111  RegD 0000  0000 0000 | MRS
//!     COND 0001  0P10 1001  1111 0000  0000 RegM | MSR by RegM
//!     COND 00I1  0P10 1000  1111 shft  shft shft | MSR by imm
//!     COND 00Ix  xxxS RegN  RegD shft  shft shft | Data Processing Op(xxxx)
//!     COND 01I+  -BWL RegN  RegD offs  offs offs | LDR/STR
//!     COND 000+  -0WL RegN  RegD 0000  1xx1 RegM | LDRH/STRH/LDRSB/LDRSH depending on Op(xx)
//!     COND 000+  -1WL RegN  RegD imm_  1xx1 imm_ | LDRH/STRH/LDRSB/LDRSH depending on Op(xx) with Offset=imm_imm_
//!     COND 100+  -RWL RegN  regs regs  regs regs | LDM/STM with register list regsregsregsregs
//!     COND 0001  0B00 RegN  RegD 0000  1001 RegM | SWP
//!     COND 1111  imm_ imm_  imm_ imm_  imm_ imm_ | SWI with comment
//!     COND 1110  CPOP CprN  CprD CPID  xxx0 CprM | CDP with CoCPU Op4 CPOP and CP Info xxx
//!     COND 1110  yyyL CprN  RegD CPID  xxx1 CprM | MRC/MCR with CoCPU Op3 yyy and CP Info xxx
//!     COND 110+  -NWL RegN  CprD CPID  imm_ imm_ | LDC/STC with unsigned Immediate
//!     COND 011?  ???? ????  ???? ????  ???1 ???? | Unknown Instruction
//! 
//! Bit Flags:
//!     I: 1=shftIsRegister,  0=shftIsImmediate
//!     F: 1=BranchWithLink,  0=BranchWithoutLink
//!     +: 1=PreIndexing,     0=PostIndexing
//!     -: 1=AddOffset,       0=SubtractOffset
//!     P: 1=SPSR,            0=CPSR
//!     U: 1=Signed,          0=Unsigned
//!     B: 1=TransferByte,    0=TransferWord
//!     R: 1=ForceUserMode,   0=NoForceUserMode
//!     N: 1=TransferAllRegs, 0=TransferSingleReg
//!     A: 1=Accumulate,      0=DoNotAccumulate
//!     W: 1=AutoIncrement,   0=NoWriteBack
//!     S: 1=SetFlags,        0=DoNotSetFlags
//!     L: 1=Load,            0=Store
//!     
//! Shift format:
//!     I=0: shft shft shft
//!          _imm _op0 RegM // RegM = RegM SHIFT(op) _imm_
//!          RegS 0op1 RegM // RegM = RegM SHIFT(op) RegS
//!          
//!     I=1: shft shft shft
//!          xxxx imm_ imm_ // Immediate = imm_imm_ ROR 2*xxxx
//! 
//! Offset format:
//!     I=0: offs offs offs
//!          imm_ imm_ imm_ // Immediate unsigned offset.
//!     
//!     I=1: offs offs offs
//!          _imm _op0 RegM // RegM = RegM SHIFT(op) _imm_
//!          RegS 0op1 RegM // RegM = RegM SHIFT(op) RegS
//! ```
//!
//! The `#map:` hints are a trick to make the code simpler.
//! For example, in the manuals, the `MUL` instruction swaps
//! the bit positions of the encoded `Rn` and `Rd` register.
//! Why? I don't know! But I do know that I find it less
//! confusing to write `Rn = Rd` instead of adding an extra
//! `Rn_mul` and `Rd_mul` register decoding function.
#![warn(missing_docs)]

use std::mem;
use std::fmt;

use super::super::error::GbaError;
use super::arm7tdmi::CPSR;


/// The condition field of an ARM instruction.
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum ArmCondition {
    #[doc = "Z set. EQual."]                                       EQ = 0b0000,
    #[doc = "Z clear. Not Equal."]                                 NE = 0b0001,
    #[doc = "C set. Unsigned Higher or Same."]                     HS = 0b0010,
    #[doc = "C clear. Unsigned LOwer."]                            LO = 0b0011,
    #[doc = "N set. MInus, i.e. negative."]                        MI = 0b0100,
    #[doc = "N clear. PLus, i.e. positive or zero."]               PL = 0b0101,
    #[doc = "V Set. Overflow."]                                    VS = 0b0110,
    #[doc = "V Clear. No Overflow."]                               VC = 0b0111,
    #[doc = "C set and Z clear. Unsigned HIgher."]                 HI = 0b1000,
    #[doc = "C clear or Z set. Unsigned Lower or Same."]           LS = 0b1001,
    #[doc = "N equals V. Greater than or Equal to."]               GE = 0b1010,
    #[doc = "N distinct from V. Less Than."]                       LT = 0b1011,
    #[doc = "Z clear and N equals V. Greater Than."]               GT = 0b1100,
    #[doc = "Z set or N distinct from V.  Less than or Equal to."] LE = 0b1101,
    #[doc = "ALways execute this instruction, i.e. no condition."] AL = 0b1110,
    #[doc = "Reserved."]                                           NV = 0b1111,
}

impl ArmCondition {
    /// Evaluates the condition field depending on the CPSR's state.
    ///
    /// # Params
    /// - `cpsr`: The CPSR to inspect.
    ///
    /// # Returns
    /// - `Ok`: `true` if the corresponding instruction should be executed, otherwise `false`.
    /// - `Err`: The condition field is `NV`, which is reserved in ARM7TDMI.
    pub fn check(self, cpsr: &CPSR) -> Result<bool, GbaError> {
        match self {
            ArmCondition::EQ => Ok( cpsr.Z() ),
            ArmCondition::NE => Ok(!cpsr.Z() ),
            ArmCondition::HS => Ok( cpsr.C() ),
            ArmCondition::LO => Ok(!cpsr.C() ),
            ArmCondition::MI => Ok( cpsr.N() ),
            ArmCondition::PL => Ok(!cpsr.N() ),
            ArmCondition::VS => Ok( cpsr.V() ),
            ArmCondition::VC => Ok(!cpsr.V() ),
            ArmCondition::HI => Ok( cpsr.C() & !cpsr.Z() ),
            ArmCondition::LS => Ok(!cpsr.C() |  cpsr.Z() ),
            ArmCondition::GE => Ok( cpsr.N() == cpsr.V() ),
            ArmCondition::LT => Ok( cpsr.N() != cpsr.V() ),
            ArmCondition::GT => Ok(!cpsr.Z() & (cpsr.N() == cpsr.V()) ),
            ArmCondition::LE => Ok( cpsr.Z() | (cpsr.N() != cpsr.V()) ),
            ArmCondition::AL => Ok( true ),
            ArmCondition::NV => Err(GbaError::ReservedArmConditionNV),
        }
    }
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
    #[doc = "Branch and change ARM/THUMB state"]      BX,
    #[doc = "Branch (with Link)"]                     B_BL,
    #[doc = "Multiply (and accumulate)"]              MUL_MLA,
    #[doc = "64-bit multiply (and accumulate)"]       MULL_MLAL,
    #[doc = "See ArmDPOP"]                            DataProcessing,
    #[doc = "Move CPSR/SPSR into a register"]         MRS,
    #[doc = "Move a register into PSR flags"]         MSR_Reg,
    #[doc = "Move an immediate into PSR flags"]       MSR_Immediate,
    #[doc = "Load/store register to/from memory"]     LDR_STR,
    #[doc = "Load/store halfwords"]                   LDRH_STRH_Reg,
    #[doc = "Load/store halfwords"]                   LDRH_STRH_Immediate,
    #[doc = "Load/store multiple registers"]          LDM_STM,
    #[doc = "Swap register with memory"]              SWP,
    #[doc = "Software interrupt with comment"]        SWI,
    #[doc = "Co-processor data processing"]           CDP,
    #[doc = "Move register to/from co-processor"]     MRC_MCR,
    #[doc = "Load/store co-processor from/to memory"] LDC_STC,
    #[doc = "Unknown instruction"]                    Unknown,
}


/// A data processing opcode.
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum ArmDPOP {
    #[doc = "Bitwise AND"]                  AND = 0b0000,
    #[doc = "Bitwise XOR"]                  EOR = 0b0001,
    #[doc = "Subtraction"]                  SUB = 0b0010,
    #[doc = "Reverse subtraction"]          RSB = 0b0011,
    #[doc = "Addition"]                     ADD = 0b0100,
    #[doc = "Add with carry"]               ADC = 0b0101,
    #[doc = "Subtract with borrow"]         SBC = 0b0110,
    #[doc = "Reverse subtract with borrow"] RSC = 0b0111,
    #[doc = "Test bits"]                    TST = 0b1000,
    #[doc = "Test bitwise equality"]        TEQ = 0b1001,
    #[doc = "Compare"]                      CMP = 0b1010,
    #[doc = "Compare negative"]             CMN = 0b1011,
    #[doc = "Bitwise OR"]                   ORR = 0b1100,
    #[doc = "Move value"]                   MOV = 0b1101,
    #[doc = "Bit clear"]                    BIC = 0b1110,
    #[doc = "Move bitwise negated value"]   MVN = 0b1111,
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
        else if (raw & 0x0E000010) == 0x06000010 { ArmOpcode::Unknown }
        else if (raw & 0x0FC000F0) == 0x00000090 { ArmOpcode::MUL_MLA }
        else if (raw & 0x0F8000F0) == 0x00800090 { ArmOpcode::MULL_MLAL }
        else if (raw & 0x0FBF0FFF) == 0x010F0000 { ArmOpcode::MRS }
        else if (raw & 0x0FBFFFF0) == 0x0129F000 { ArmOpcode::MSR_Reg }
        else if (raw & 0x0DBFF000) == 0x0128F000 { ArmOpcode::MSR_Immediate }
        else if (raw & 0x0FB00FF0) == 0x01000090 { ArmOpcode::SWP }
        else if (raw & 0x0C000000) == 0x04000000 { ArmOpcode::LDR_STR }
        else if (raw & 0x0E400F90) == 0x00000090 { ArmOpcode::LDRH_STRH_Reg }
        else if (raw & 0x0E400090) == 0x00400090 { ArmOpcode::LDRH_STRH_Immediate }
        else if (raw & 0x0E000000) == 0x08000000 { ArmOpcode::LDM_STM }
        else if (raw & 0x0F000000) == 0x0F000000 { ArmOpcode::SWI }
        else if (raw & 0x0F000010) == 0x0E000000 { ArmOpcode::CDP }
        else if (raw & 0x0F000010) == 0x0E000010 { ArmOpcode::MRC_MCR }
        else if (raw & 0x0E000000) == 0x0C000000 { ArmOpcode::LDC_STC }
        else if (raw & 0x0C000000) == 0x00000000 { ArmOpcode::DataProcessing }
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
        let off = (self.raw & 0xFF) as i32;
        if self.is_offset_added() { off } else { -off }
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
        ((self.raw & 0xFF) as u32).rotate_right(bits) as i32
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
    
    
    // Below here is just a bunch of
    // messy functions to display an
    // instruction disassembly on demand.
    
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
            let off: i32 = self.raw & 0x0FFF;
            try!(write!(f, "#{}", if self.is_offset_added() { off } else { -off }));
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
    
    fn display_register_list(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{{"));
        for i in 0 .. 16 {
            if (self.raw & (1 << i)) != 0 { try!(write!(f, "R{}, ", i)); }
        }
        write!(f, "}}{}", if self.is_enforcing_user_mode() { "^" } else { "" })
    }
}



impl fmt::Display for ArmInstruction {
    /// Writes a disassembly of the given instruction to a formatter.
    ///
    /// # Params
    /// - `f`: The formatter to write to.
    ///
    /// # Returns
    /// - `Ok` if everything succeeded.
    /// - `Err` in case of an error.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{:#010X}\t", self.raw as u32));
        
        let cond = self.condition();
        
        match self.op {
            ArmOpcode::Unknown        => write!(f, "<unknown>"),
            ArmOpcode::SWI            => write!(f, "swi{}\t#{:#08X}", cond, self.comment()),
            ArmOpcode::BX             => write!(f, "bx{}\tR{}", cond, self.Rm()),
            ArmOpcode::B_BL           => write!(f, "b{}{}\t#{}", if self.is_branch_with_link() { "l" } else { "" }, cond, 8+self.branch_offset()),
            ArmOpcode::MRS            => write!(f, "mrs{}\tR{}, {}", cond, self.Rd(), if self.is_accessing_spsr() { "SPSR" } else { "CPSR" }),
            ArmOpcode::MSR_Reg        => write!(f, "msr{}\t{}, R{}", cond, if self.is_accessing_spsr() { "SPSR" } else { "CPSR" }, self.Rm()),
            ArmOpcode::LDRH_STRH_Reg  => write!(f, "{}{}{}\tR{}, [R{}{}, {}R{}{}{}",
                if self.is_load() { "ldr" } else { "str" },
                match (self.raw >> 5) & 0b11 {
                    1 => "h",
                    2 => "sb",
                    3 => "sh",
                    _ => unimplemented!(),
                }, cond,
                self.Rd(), self.Rn(), if self.is_pre_indexed() { "" } else { "]" },
                if self.is_offset_added() { "" } else { "+" }, self.Rm(),
                if self.is_pre_indexed() { "]" } else { "" },
                if self.is_auto_incrementing() { "!" } else { "" }
            ),
            ArmOpcode::LDRH_STRH_Immediate => write!(f, "{}{}{}\tR{}, [R{}{}, #{}{}{}{}",
                if self.is_load() { "ldr" } else { "str" },
                match (self.raw >> 5) & 0b11 {
                    1 => "h",
                    2 => "sb",
                    3 => "sh",
                    _ => unimplemented!(),
                }, cond,
                self.Rd(), self.Rn(), if self.is_pre_indexed() { "" } else { "]" },
                if self.is_offset_added() { "" } else { "+" }, self.split_offset8(),
                if self.is_pre_indexed() { "]" } else { "" },
                if self.is_auto_incrementing() { "!" } else { "" }
            ),
            ArmOpcode::LDR_STR => {
                try!(write!(f, "{}{}{}{}\tR{}, ",
                    if self.is_load() { "ldr" } else { "str" },
                    if self.is_transfering_bytes() { "b" } else { "" }, cond,
                    if self.is_pre_indexed() { "" } else { "t" },
                    self.Rd()
                ));
                self.display_offset(f)
            },
            ArmOpcode::LDM_STM => {
                try!(write!(f, "{}{}{}{}\tR{}{}, ",
                    if self.is_load() { "ldm" } else { "stm" },
                    if self.is_offset_added() { "i" } else { "d" },
                    if self.is_pre_indexed()  { "b" } else { "a" },
                    cond, self.Rn(),
                    if self.is_auto_incrementing() { "!" } else { "" }
                ));
                self.display_register_list(f)
            },
            ArmOpcode::SWP => write!(f, "swp{}{}\tR{}, R{}, [R{}]",
                if self.is_transfering_bytes() { "b" } else { "" },
                cond, self.Rd(), self.Rm(), self.Rn()
            ),
            ArmOpcode::MUL_MLA => {
                try!(write!(f, "{}{}{}\tR{}, R{}, R{}",
                    if self.is_accumulating() { "mla" } else { "mul" },
                    if self.is_setting_flags() { "s" } else { "" }, cond,
                    self.Rn(), self.Rm(), self.Rs(),
                ));
                if self.is_accumulating() {
                    write!(f, ", R{}", self.Rd())
                }
                else { Ok(()) }
            },
            ArmOpcode::MULL_MLAL => write!(f, "{}{}{}{}\tR{}, R{}, R{}, R{}",
                if self.is_signed() { "s" } else { "u" },
                if self.is_accumulating() { "mlal" } else { "mull" },
                if self.is_setting_flags() { "s" } else { "" }, cond,
                self.Rd(), self.Rn(), self.Rm(), self.Rs(),
            ),
            ArmOpcode::MSR_Immediate => {
                try!(write!(f, "msr{}\t{}, ", cond, if self.is_accessing_spsr() { "SPSR_flg" } else { "CPSR_flg" }));
                if !self.is_shift_field_register() { write!(f, "#{:#010X}", self.rotated_immediate() as u32) }
                else { write!(f, "R{}", self.Rm()) }
            },
            ArmOpcode::DataProcessing => {
                let op = self.dpop();
                try!(write!(f, "{}{}{}\t", &op, cond, if self.is_setting_flags() && !op.is_test() { "s" } else { "" }));
                if !op.is_test() { try!(write!(f, "R{}, ", self.Rd())); }
                if !op.is_move() { try!(write!(f, "R{}, ", self.Rn())); }
                self.display_shift(f)
            },
            ArmOpcode::CDP => write!(f, "cdp{}\tP{}, {}, C{}, C{}, C{}, {}",
                cond, self.Rs(), (self.raw >> 20) & 0b1111, self.Rd(),
                self.Rn(), self.Rm(), (self.raw >> 5) & 0b0111
            ),
            ArmOpcode::LDC_STC => write!(f, "{}{}{}\tP{}, C{}, [R{}{}, #{}{}{}",
                if self.is_load() { "ldc" } else { "stc" },
                if self.is_register_block_transfer() { "l" } else { "" },
                cond, self.Rs(), self.Rd(), self.Rn(),
                if self.is_pre_indexed() { "" } else { "]" },
                self.offset8(),
                if self.is_pre_indexed() { "]" } else { "" },
                if self.is_auto_incrementing() { "!" } else { "" }
            ),
            ArmOpcode::MRC_MCR => write!(f, "{}{}\tP{}, {}, R{}, C{}, C{}, {}",
                if self.is_load() { "mrc" } else { "mcr" }, cond, self.Rs(),
                (self.raw >> 21) & 0b0111, self.Rd(), self.Rn(), self.Rm(),
                (self.raw >>  5) & 0b0111
            ),
        }
    }
}



#[cfg(test)]
mod test {
    use std::fmt::Write;
    
    pub const INSTRUCTIONS_RAW: &'static [i32] = &[
        // Use SWI to check condition decoding.
        0b0000_1111_011101110111011101110111_u32 as i32,
        0b0001_1111_011101110111011101110111_u32 as i32,
        0b0010_1111_011101110111011101110111_u32 as i32,
        0b0011_1111_011101110111011101110111_u32 as i32,
        0b0100_1111_011101110111011101110111_u32 as i32,
        0b0101_1111_011101110111011101110111_u32 as i32,
        0b0110_1111_011101110111011101110111_u32 as i32,
        0b0111_1111_011101110111011101110111_u32 as i32,
        0b1000_1111_011101110111011101110111_u32 as i32,
        0b1001_1111_011101110111011101110111_u32 as i32,
        0b1010_1111_011101110111011101110111_u32 as i32,
        0b1011_1111_011101110111011101110111_u32 as i32,
        0b1100_1111_011101110111011101110111_u32 as i32,
        0b1101_1111_011101110111011101110111_u32 as i32,
        0b1110_1111_011101110111011101110111_u32 as i32,
        0b1111_1111_011101110111011101110111_u32 as i32,
        
        // Test BX, B, BL.
        0b0000_000100101111111111110001_0111_u32 as i32,
        0b0000_101_0_111111111111111111111101_u32 as i32,
        0b0000_101_0_000000000000000000000001_u32 as i32,
        0b0000_101_1_111111111111111111111101_u32 as i32,
        0b0000_101_1_000000000000000000000001_u32 as i32,
        
        // Test Unknown.
        0b0000_011_01100110011001100110_1_0110_u32 as i32,
        
        // Data Processing.
        0b0000_00_1_0000_0_0001_0010_0011_01000101_u32 as i32,
        0b0000_00_0_0001_1_0001_0010_00111_00_0_0011_u32 as i32,
        0b0000_00_0_0010_0_0001_0010_00111_01_0_0011_u32 as i32,
        0b0000_00_0_0011_0_0001_0010_00111_10_0_0011_u32 as i32,
        0b0000_00_0_0100_0_0001_0010_00111_11_0_0011_u32 as i32,
        0b0000_00_0_0101_0_0001_0010_00000_11_0_0011_u32 as i32,
        0b0000_00_0_0110_0_0001_0010_00000_00_0_0011_u32 as i32,
        0b0000_00_0_0111_0_0001_0010_0100_0_00_1_0011_u32 as i32,
        0b0000_00_0_1000_0_0001_0010_0100_0_01_1_0011_u32 as i32,
        0b0000_00_0_1001_0_0001_0010_0100_0_10_1_0011_u32 as i32,
        0b0000_00_0_1010_0_0001_0010_0100_0_11_1_0011_u32 as i32,
        0b0000_00_0_1011_0_0001_0010_00000_00_0_0011_u32 as i32,
        0b0000_00_0_1100_0_0001_0010_00000_00_0_0011_u32 as i32,
        0b0000_00_0_1101_0_0001_0010_00000_00_0_0011_u32 as i32,
        0b0000_00_0_1110_0_0001_0010_00000_00_0_0011_u32 as i32,
        0b0000_00_0_1111_0_0001_0010_00000_00_0_0011_u32 as i32,
        
        // MRS and MSR.
        0b0000_00010_0_001111_0001_000000000000_u32 as i32,
        0b0000_00010_1_001111_0001_000000000000_u32 as i32,
        0b0000_00010_0_101001111100000000_0010_u32 as i32,
        0b0000_00_0_10_0_1010001111_00000000_0111_u32 as i32,
        0b0000_00_0_10_1_1010001111_11111111_0111_u32 as i32,
        0b0000_00_1_10_0_1010001111_0010_00001111_u32 as i32,
    ];
    
    pub const EXPECTED_DISASSEMBLY: &'static str = "\
        0x0F777777\tswieq\t#0x777777\n\
        0x1F777777\tswine\t#0x777777\n\
        0x2F777777\tswihs\t#0x777777\n\
        0x3F777777\tswilo\t#0x777777\n\
        0x4F777777\tswimi\t#0x777777\n\
        0x5F777777\tswipl\t#0x777777\n\
        0x6F777777\tswivs\t#0x777777\n\
        0x7F777777\tswivc\t#0x777777\n\
        0x8F777777\tswihi\t#0x777777\n\
        0x9F777777\tswils\t#0x777777\n\
        0xAF777777\tswige\t#0x777777\n\
        0xBF777777\tswilt\t#0x777777\n\
        0xCF777777\tswigt\t#0x777777\n\
        0xDF777777\tswile\t#0x777777\n\
        0xEF777777\tswi\t#0x777777\n\
        0xFF777777\tswinv\t#0x777777\n\
        0x012FFF17\tbxeq\tR7\n\
        0x0AFFFFFD\tbeq\t#-4\n\
        0x0A000001\tbeq\t#12\n\
        0x0BFFFFFD\tbleq\t#-4\n\
        0x0B000001\tbleq\t#12\n\
        0x06CCCCD6\t<unknown>\n\
        0x02012345\tandeq\tR2, R1, #335544321\n\
        0x00312383\teoreqs\tR2, R1, R3, lsl #7\n\
        0x004123A3\tsubeq\tR2, R1, R3, lsr #7\n\
        0x006123C3\trsbeq\tR2, R1, R3, asr #7\n\
        0x008123E3\taddeq\tR2, R1, R3, ror #7\n\
        0x00A12063\tadceq\tR2, R1, R3, rrx\n\
        0x00C12003\tsbceq\tR2, R1, R3\n\
        0x00E12413\trsceq\tR2, R1, R3, lsl R4\n\
        0x01012433\ttsteq\tR1, R3, lsr R4\n\
        0x01212453\tteqeq\tR1, R3, asr R4\n\
        0x01412473\tcmpeq\tR1, R3, ror R4\n\
        0x01612003\tcmneq\tR1, R3\n\
        0x01812003\torreq\tR2, R1, R3\n\
        0x01A12003\tmoveq\tR2, R3\n\
        0x01C12003\tbiceq\tR2, R1, R3\n\
        0x01E12003\tmvneq\tR2, R3\n\
        0x010F1000\tmrseq\tR1, CPSR\n\
        0x014F1000\tmrseq\tR1, SPSR\n\
        0x0129F002\tmsreq\tCPSR, R2\n\
        0x0128F007\tmsreq\tCPSR_flg, R7\n\
        0x0168FFF7\tmsreq\tSPSR_flg, R7\n\
        0x0328F20F\tmsreq\tCPSR_flg, #0xF0000000\n\
    ";
    
    #[test]
    pub fn instruction_disassembly() {
        let mut dis = String::new();
        
        for inst in self::INSTRUCTIONS_RAW {
            writeln!(dis, "{}", super::ArmInstruction::decode(*inst).unwrap()).unwrap();
        }
        
        println!("\n========================\nGenerated Disassembly:\n\n{}", dis);
        println!("\n========================\nExpected Disassembly:\n\n{}", self::EXPECTED_DISASSEMBLY);
        
        assert!(dis == self::EXPECTED_DISASSEMBLY);
    }
}


/*
Licensed to the Apache Software Foundation (ASF) under one
or more contributor license agreements.  See the NOTICE file
distributed with this work for additional information
regarding copyright ownership.  The ASF licenses this file
to you under the Apache License, Version 2.0 (the
"License"); you may not use this file except in compliance
with the License.  You may obtain a copy of the License at

  http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing,
software distributed under the License is distributed on an
"AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
KIND, either express or implied.  See the License for the
specific language governing permissions and limitations
under the License.
*/
