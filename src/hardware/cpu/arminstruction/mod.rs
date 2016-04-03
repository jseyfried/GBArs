// License below.
//! Implements utilities to decode, disassemble, and handle
//! 32-bit ARM state instructions.
//!
//! These tables show how ARM state instructions are encoded:
//!
//! ```text
//! Full Instructions:
//!     .... ....  .... ....  .... ....  .... ....
//!     COND 0001  0010 1111  1111 1111  0001 RegM | BX #map: RegN => RegM
//!     COND 101F  imm_ imm_  imm_ imm_  imm_ imm_ | B/BL with signed offset
//!     COND 0000  00AS RegN  RegD RegS  1001 RegM | MUL/MLA #map: RegN <=> RegD
//!     COND 0000  1UAS RegN  RegD RegS  1001 RegM | MULL/MLAL #map: RdHi => RegN, RdLo => RegD
//!     COND 0001  0P00 1111  RegD 0000  0000 0000 | MRS
//!     COND 0001  0P10 1001  1111 0000  0000 RegM | MSR by RegM
//!     COND 00I1  0P10 1000  1111 shsr  shsr shsr | MSR into flags
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
//! MSR shift format:
//!     I=0: shsr shsr shsr
//!          0000 0000 RegM // RegM = RegM
//!
//!     I=1: shsr shsr shsr
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
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use std::mem;

use super::super::error::GbaError;
use super::arm7tdmi::{Arm7Tdmi, ArmCondition};
use super::arm7tdmi::exec::armdpop::ArmDPOP;

pub use self::display::*;

mod display;

#[cfg(test)]
mod test;

/// A special opcode for halfword and signed data transfers.
#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum ArmLdrhStrhOP {
    #[doc = "Decoded LDRH/STRH instead of SWP."] InvalidSWP = 0,
    #[doc = "Unsigned halfword load/store."]     UH = 1,
    #[doc = "Signed byte load/store."]           SB = 2,
    #[doc = "Signed halfword load/store"]        SH = 3,
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
    #[doc = "Move an immediate into PSR flags"]       MSR_Flags,
    #[doc = "Load/store register to/from memory"]     LDR_STR,
    #[doc = "Load/store halfwords"]                   LDRH_STRH_Reg,
    #[doc = "Load/store halfwords"]                   LDRH_STRH_Imm,
    #[doc = "Load/store multiple registers"]          LDM_STM,
    #[doc = "Swap register with memory"]              SWP,
    #[doc = "Software interrupt with comment"]        SWI,
    #[doc = "Co-processor data processing"]           CDP,
    #[doc = "Move register to/from co-processor"]     MRC_MCR,
    #[doc = "Load/store co-processor from/to memory"] LDC_STC,
    #[doc = "Unknown instruction"]                    Unknown,
}

/// A decoded ARM instruction providing lots
/// of utility and decoding functions to ease
/// ARM instruction emulation.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ArmInstruction {
    raw: u32,
    op: ArmOpcode,
}

impl ArmInstruction {
    /// A raw 32-bit pseudo NOP instruction.
    pub const NOP_RAW: u32 = 0b0000_00_0_1101_0_0000_0000_00000000_0000_u32;
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
    pub fn decode(raw: u32) -> Result<ArmInstruction, GbaError> {
        // Decode the opcode to something easier to match and read.
        let op: ArmOpcode =
             if (raw & 0x0FFFFFF0) == 0x012FFF10 { ArmOpcode::BX }
        else if (raw & 0x0E000000) == 0x0A000000 { ArmOpcode::B_BL }
        else if (raw & 0x0E000010) == 0x06000010 { ArmOpcode::Unknown }
        else if (raw & 0x0FB00FF0) == 0x01000090 { ArmOpcode::SWP }
        else if (raw & 0x0FC000F0) == 0x00000090 { ArmOpcode::MUL_MLA }
        else if (raw & 0x0F8000F0) == 0x00800090 { ArmOpcode::MULL_MLAL }
        else if (raw & 0x0FBF0FFF) == 0x010F0000 { ArmOpcode::MRS } // Order matters here, as...
        else if (raw & 0x0FBFFFF0) == 0x0129F000 { ArmOpcode::MSR_Reg } // ... these here are subsets...
        else if (raw & 0x0DBFF000) == 0x0128F000 { ArmOpcode::MSR_Flags } // ... of DataProcessing.
        else if (raw & 0x0C000000) == 0x04000000 { ArmOpcode::LDR_STR }
        else if (raw & 0x0E400F90) == 0x00000090 { ArmOpcode::LDRH_STRH_Reg }
        else if (raw & 0x0E400090) == 0x00400090 { ArmOpcode::LDRH_STRH_Imm }
        else if (raw & 0x0E000000) == 0x08000000 { ArmOpcode::LDM_STM }
        else if (raw & 0x0F000000) == 0x0F000000 { ArmOpcode::SWI }
        else if (raw & 0x0F000010) == 0x0E000000 { ArmOpcode::CDP }
        else if (raw & 0x0F000010) == 0x0E000010 { ArmOpcode::MRC_MCR }
        else if (raw & 0x0E000000) == 0x0C000000 { ArmOpcode::LDC_STC }
        else if (raw & 0x0C000000) == 0x00000000 { ArmOpcode::DataProcessing }
        else { return Err(GbaError::InvalidArmInstruction(raw)); };

        // Done decoding!
        Ok(ArmInstruction { raw: raw, op: op })
    }

    /// Checks an instruction's validity.
    ///
    /// Some instructions have constraints on how to use them,
    /// e.g. some forbid to use PC as a destination register.
    ///
    /// ## Returns
    /// - `Ok`: The instruction might have unpredictable side effects, but it is valid.
    /// - `Err`: The instruction violates the CPU's constraints.
    pub fn check_is_valid(&self) -> Result<(), GbaError> {
        let pc = Arm7Tdmi::PC; let rn = self.Rn(); let rd = self.Rd(); let rs = self.Rs(); let rm = self.Rm();

        // Check for an invalid use of PC.
        if rn==pc {
            if self.is_auto_incrementing() { match self.op {
                ArmOpcode::LDC_STC | ArmOpcode::LDR_STR | ArmOpcode::LDRH_STRH_Reg | ArmOpcode::LDRH_STRH_Imm => {
                    return Err(GbaError::InvalidUseOfR15);
                }, _ => {},
            }} else { match self.op {
                ArmOpcode::SWP | ArmOpcode::MUL_MLA | ArmOpcode::MULL_MLAL | ArmOpcode::LDM_STM => {
                    return Err(GbaError::InvalidUseOfR15);
                }, _ => {},
            }}
        }
        if rd==pc { match self.op {
            ArmOpcode::MRS | ArmOpcode::SWP | ArmOpcode::MUL_MLA | ArmOpcode::MULL_MLAL => {
                return Err(GbaError::InvalidUseOfR15)
            },
            _ => {},
        }}
        if rs==pc { match self.op {
            ArmOpcode::SWP | ArmOpcode::MUL_MLA | ArmOpcode::MULL_MLAL => { return Err(GbaError::InvalidUseOfR15); },
            _ => {},
        }}
        if rm==pc { match self.op {
            ArmOpcode::MSR_Reg | ArmOpcode::MUL_MLA | ArmOpcode::MULL_MLAL |
            ArmOpcode::LDR_STR | ArmOpcode::LDRH_STRH_Reg | ArmOpcode::LDRH_STRH_Imm => {
                return Err(GbaError::InvalidUseOfR15);
            },
            _ => {},
        }}

        // Check register usage for multiplication.
        if rd==rm && (
            (self.op == ArmOpcode::MUL_MLA)
        || ((self.op == ArmOpcode::MULL_MLAL) && ((rn==rm) | (rn==rd)))
        ) { return Err(GbaError::InvalidRegisterReuse(rn,rd,rs,rm)); }

        // Check valid write-back.
        match self.op {
            ArmOpcode::LDRH_STRH_Reg | ArmOpcode::LDRH_STRH_Imm => {
                if self.is_auto_incrementing() & !self.is_pre_indexed() { Err(GbaError::InvalidOffsetWriteBack) }
                else { Ok(()) }
            },
            ArmOpcode::LDM_STM => {
                let has15 = 0 != (self.raw & 0x8000);
                if !(self.is_load() & has15) & self.is_enforcing_user_mode() & self.is_auto_incrementing() {
                    Err(GbaError::InvalidOffsetWriteBack)
                } else { Ok(()) }
            },
            _ => Ok(()),
        }
    }

    /// Get the condition field of the ARM instruction.
    pub fn condition(&self) -> ArmCondition {
        let c = ((self.raw >> 28) & 0b1111) as u8;
        unsafe { mem::transmute(c) }
    }

    /// Get the decoded opcode of the ARM instruction.
    pub fn opcode(&self) -> ArmOpcode {
        self.op
    }

    /// Get the data processing opcode field of the ARM instruction.
    pub fn dpop(&self) -> ArmDPOP {
        let o = ((self.raw >> 21) & 0b1111) as u8;
        unsafe { mem::transmute(o) }
    }

    /// Get the LDRH/STRH opcode field of the ARM instruction.
    pub fn ldrh_strh_op(&self) -> ArmLdrhStrhOP {
        let o = ((self.raw >> 5) & 0b11) as u8;
        unsafe { mem::transmute(o) }
    }

    /// Get the index of register `Rn`.
    #[allow(non_snake_case)]
    pub fn Rn(&self) -> usize { ((self.raw >> 16) & 0b1111) as usize }

    /// Get the index of register `Rd`.
    #[allow(non_snake_case)]
    pub fn Rd(&self) -> usize { ((self.raw >> 12) & 0b1111) as usize }

    /// Get the index of register `Rs`.
    #[allow(non_snake_case)]
    pub fn Rs(&self) -> usize { ((self.raw >> 8) & 0b1111) as usize }

    /// Get the index of register `Rm`.
    #[allow(non_snake_case)]
    pub fn Rm(&self) -> usize { ((self.raw     ) & 0b1111) as usize }

    /// Get the target co-processor's ID.
    #[cfg_attr(feature="clippy", allow(inline_always))]
    #[inline(always)]
    pub fn cp_id(&self) -> usize { self.Rs() }

    /// Get a 4-bit opcode for the target co-processor.
    pub fn cp_opcode4(&self) -> u8 { ((self.raw >> 20) & 0b1111) as u8 }

    /// Get a 3-bit opcode for the target co-processor.
    pub fn cp_opcode3(&self) -> u8 { ((self.raw >> 21) & 0b0111) as u8 }

    /// Get a 3-bit info number for the target co-processor.
    pub fn cp_info(&self) -> u8 { ((self.raw >> 5) & 0b0111) as u8 }

    /// Gets the shift value for a shifted register.
    pub fn register_shift_immediate(&self) -> u32 { (self.raw >> 7) & 0b1_1111 }

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
        if self.is_offset_field_immediate() { self.offset12() }
        else {
            let offs = self.calculate_shifted_register(regs, carry);
            if self.is_offset_added() { offs } else { -offs }
        }
    }

    /// Gets a zero-extended 12-bit immediate to be used with LDR/STR.
    pub fn offset12(&self) -> i32 {
        let off = (self.raw & 0xFFF) as i32;
        if self.is_offset_added() { off } else { -off }
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
        let off = (((self.raw >> 4) & 0xF0) | (self.raw & 0x0F)) as i32;
        if self.is_offset_added() { off } else { -off }
    }

    /// Get the 24-bit sign-extended branch offset.
    pub fn branch_offset(&self) -> i32 { ((self.raw << 8) as i32) >> 6 }

    /// Get the 24-bit comment field of an `SWI` instruction.
    pub fn comment(&self) -> u32 { self.raw & 0x00FFFFFF }

    /// Get a 16-bit bitmap, where bit N corresponds to GPR N.
    pub fn register_map(&self) -> u16 { (self.raw & 0xFFFF) as u16 }

    /// Determines whether a shift field is to be decoded as
    /// rotated immediate value or as a shifted register value.
    ///
    /// # Returns
    /// - `true`: Shift field is a register shift.
    /// - `false`: Shift field is a rotated immediate.
    pub fn is_shift_field_register(&self) -> bool { (self.raw & (1 << 25)) == 0 }

    /// Decodes a rotated immediate value.
    ///
    /// # Returns
    /// An immediate 32-bit value consisting of a single
    /// rotated byte.
    pub fn rotated_immediate(&self) -> i32 {
        let bits = 2 * ((self.raw >> 8) & 0b1111);
        (self.raw & 0xFF).rotate_right(bits) as i32
    }

    /// Determines whether an offset field is to be decoded
    /// as shifted registers or as an immediate value.
    ///
    /// # Returns
    /// - `true`: The offset is an immediate non-sign-extended value.
    /// - `false`: The offset is a shifted register value.
    #[cfg_attr(feature="clippy", allow(inline_always))]
    #[inline(always)]
    pub fn is_offset_field_immediate(&self) -> bool { self.is_shift_field_register() }

    /// Checks whether this is a `B` or `BL` instruction.
    ///
    /// # Returns:
    /// - `true`: `BL`
    /// - `false`: `B`
    pub fn is_branch_with_link(&self) -> bool { (self.raw & (1 << 24)) != 0 }

    /// Checks whether an offset register should be
    /// pre-indexed or post-indexed.
    ///
    /// # Returns:
    /// - `true`: pre-indexed
    /// - `false`: post-indexed
    pub fn is_pre_indexed(&self) -> bool { (self.raw & (1 << 24)) != 0 }

    /// Checks whether a given offset should be added
    /// or subtracted from a base address.
    ///
    /// # Returns
    /// - `true`: Add the given offset to the base address.
    /// - `false`: Subtract the given offset from the base address.
    pub fn is_offset_added(&self) -> bool { (self.raw & (1 << 23)) != 0 }

    /// Checks whether the given instruction accesses CPSR
    /// or the current SPSR.
    ///
    /// # Returns
    /// - `true`: Accessing the current SPSR.
    /// - `false`: Accessing CPSR.
    pub fn is_accessing_spsr(&self) -> bool { (self.raw & (1 << 22)) != 0 }

    /// Checks whether the given long instruction should
    /// act as a signed or unsigned operation.
    ///
    /// # Returns
    /// - `true`: The operation is signed.
    /// - `false`: The operation is unsigned.
    pub fn is_signed(&self) -> bool { (self.raw & (1 << 22)) != 0 }

    /// Checks whether a data transfer instruction should
    /// transfer bytes or words.
    ///
    /// # Returns
    /// - `true`: Transfer bytes.
    /// - `false`: Transfering words.
    pub fn is_transfering_bytes(&self) -> bool { (self.raw & (1 << 22)) != 0 }

    /// Checks whether register block transfer should be
    /// done in user mode.
    ///
    /// # Returns
    /// - `true`: Enforce user mode for privileged code.
    /// - `false`: Execute in current mode.
    pub fn is_enforcing_user_mode(&self) -> bool { (self.raw & (1 << 22)) != 0 }

    /// Checks whether a single register or a block of
    /// registers should be transfered to or from a
    /// co-processor.
    ///
    /// # Returns
    /// - `true`: Transfer a block of registers.
    /// - `false`: Transfer a single register.
    pub fn is_register_block_transfer(&self) -> bool { (self.raw & (1 << 22)) != 0 }

    /// Checks whether a multiply instruction should
    /// accumulate or not.
    ///
    /// # Returns
    /// - `true`: Accumulate.
    /// - `false`: Don't accumulate.
    pub fn is_accumulating(&self) -> bool { (self.raw & (1 << 21)) != 0 }

    /// Checks whether the current instruction writes
    /// a calculated address back to the base register.
    pub fn is_auto_incrementing(&self) -> bool { (self.raw & (1 << 21)) != 0 }

    /// Checks whether the given instruction updates the
    /// ZNCV status flags of CPSR.
    ///
    /// # Returns
    /// - `true`: Updates CPSR.
    /// - `false`: Does not modify CPSR.
    pub fn is_setting_flags(&self) -> bool { (self.raw & (1 << 20)) != 0 }

    /// Checks whether the given instruction is a
    /// load or store instruction.
    ///
    /// # Returns
    /// - `true`: Load instruction.
    /// - `false`: Store instruction.
    pub fn is_load(&self) -> bool { (self.raw & (1 << 20)) != 0 }

    /// Cheks whether the register shift field is an
    /// immediate or shift register.
    ///
    /// # Returns
    /// - `true`: Shift `Rm` by an immediate.
    /// - `false`: Shift `Rm` by `Rs`.
    pub fn is_register_shift_immediate(&self) -> bool { (self.raw & (1 << 4)) == 0 }

    /// Calculates the result of a PSR transfer shift
    /// field operand.
    ///
    /// # Params
    /// - `regs`: A reference to the CPU's general purpose registers.
    ///
    /// # Returns
    /// A ready-to-use operand.
    pub fn calculate_shsr_field(&self, regs: &[i32]) -> i32 {
        if self.is_shift_field_register() { regs[self.Rm()] }
        else { self.rotated_immediate() }
    }

    /// Calculates the result of a shift field operand
    /// without calculating the carry flag.
    ///
    /// # Params
    /// - `regs`: A reference to the CPU's general purpose registers.
    /// - `carry`: The current state of the carry flag. Used for RRX.
    ///
    /// # Returns
    /// A ready-to-use operand.
    pub fn calculate_shft_field(&self, regs: &[i32], carry: bool) -> i32 {
        if self.is_shift_field_register() { self.calculate_shifted_register(regs, carry) }
        else { self.rotated_immediate() }
    }

    /// Calculates a shifted register operand without
    /// calculating the carry flag.
    ///
    /// # Params
    /// - `regs`: A reference to the CPU's general purpose registers.
    /// - `carry`: The current state of the carry flag. Used for RRX.
    ///
    /// # Returns
    /// A ready-to-use operand.
    pub fn calculate_shifted_register(&self, regs: &[i32], carry: bool) -> i32 { // FIXME Configure handling Rs.
        let a = regs[self.Rm()];

        // Handle special shifts.
        if (self.raw & 0x0F90) == 0 { match (self.raw >> 5) & 0b11 {
            0 => { return a; },
            1 => { return 0; },
            2 => { return a >> 31; },
            3 => { // RRX
                let bit31: i32 = if carry { 0x80000000_u32 as i32 } else { 0 };
                return bit31 | (((a as u32) >> 1) as i32);
            },
            _ => unreachable!(),
        }}

        let b: u32 = if (self.raw & (1 << 4)) == 0 {
            (self.raw >> 7) & 0b1_1111
        } else {
            match self.decode_Rs_shift(a, regs) { Ok(x) => x, Err(y) => { return y; }, }
        };

        match (self.raw >> 5) & 0b11 {
            0 => a.wrapping_shl(b),
            1 => (a as u32).wrapping_shr(b) as i32,
            2 => a.wrapping_shr(b),
            3 => a.rotate_right(b % 32),
            _ => unreachable!(),
        }
    }

    /// Calculates the result of a shift field and
    /// calculates the resulting carry flag.
    ///
    /// # Params
    /// - `regs`: A reference to the CPU's general purpose registers.
    /// - `carry`: The current state of the carry flag. Used for RRX.
    ///
    /// # Returns
    /// - `.0`: A ready-to-use operand.
    /// - `.1`: `true` if carry, otherwise `false`.
    pub fn calculate_shft_field_with_carry(&self, regs: &[i32], carry: bool) -> (i32, bool) {
        if self.is_shift_field_register() { self.calculate_shifted_register_with_carry(regs, carry) }
        else { (self.rotated_immediate(), false) }
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
        let a = regs[self.Rm()];

        // Handle special shifts.
        if (self.raw & 0x0F90) == 0 { match (self.raw >> 5) & 0b11 {
            0 => { return (a, carry); },
            1 => { return (0, a < 0); },
            2 => { return (a >> 31, a < 0); },
            3 => { // RRX
                let bit31: i32 = if carry { 0x80000000_u32 as i32 } else { 0 };
                return (bit31 | (((a as u32) >> 1) as i32), 0 != (a & 0b1));
            },
            _ => unimplemented!(),
        }}

        // A shift by Rs==0 just returns `a` without a new carry flag.
        let b: u32 = if (self.raw & (1 << 4)) == 0 {
            (self.raw >> 7) & 0b1_1111
        } else {
            match self.decode_Rs_shift_with_carry(a, regs, carry) { Ok(x) => x, Err(y) => { return y; }, }
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

        if r >= 32 { Err(match (self.raw >> 5) & 0b11 {
            0 | 1 => 0,
            2 => a >> 31,
            3 => if r == 32 { a } else { a.rotate_right(r % 32) },
            _ => unreachable!(),
        })}
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
}

impl Default for ArmInstruction {
    /// Creates a NOP instruction.
    fn default() -> ArmInstruction { ArmInstruction::nop() }
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
