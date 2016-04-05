// License below.
//! Implements utilities to decode, disassemble, and handle
//! 16-bit THUMB state instructions.
//!
//! These tables show how THUMB state instructions are encoded:
//!
//! ```text
//! Full Instructions:
//!     .... ....  .... ....
//!     0001 1IA<  N><S ><D> | Add/Sub, Rd = Rs OP(A) Rn
//!     000o p_im  m_<S ><D> | Move Shift, Rd = Rs SHIFT(op) #_imm_
//!     001o p<M>  imm_ imm_ | Data Processing, Rm = Rm OP(op) #imm_imm_
//!     0100 0011  01<S ><D> | MUL Rd, Rs
//!     0100 00_o  p_<S ><D> | ALU Operation, Rd = Rd OP(_op_) Rs
//!     0100 01op  hH<S ><D> | Hi-Reg Op / BX, Rd/Hd = Rd/Hd OP(op) Rs/Hs
//!     0100 1<M>  imm_ imm_ | LDR Rm, [PC, #imm_imm_00]
//!     0101 LB0<  N><S ><D> | LDR/STR Rd, [Rs, Rn]
//!     0101 WS1<  N><S ><D> | LDRH/STRH Rd, [Rs, Rn]
//!     011b L_im  m_<S ><D> | LDR/STR Rd, [Rs, #_imm_]
//!     1000 L_im  m_<S ><D> | LDRH/STRH Rd, [Rs, #_imm_]
//!     1001 L<M>  imm_ imm_ | LDR/STR Rm, [SP, #imm_imm_00]
//!     1010 P<M>  imm_ imm_ | ADD Rm, PC/SP, #imm_imm_00
//!     1011 0000  offs offs | ADD SP, SP, #SignExtend(offsoffs0)
//!     1011 L10R  regs regs | PUSH/POP regsregs
//!     1100 L<M>  regs regs | LDM/STM Rm, regsregs
//!     1101 1111  comm ent_ | SWI comment_
//!     1101 cond  offs offs | B{cond} #SignExtend(offsoffs0)
//!     1110 0off  offs offs | B #SignExtend(offoffsoffs0)
//!     1111 Xoff  offs offs | BL #Offset23Bit
//!
//! Bit Flags:
//!     I: 1=RnImmediateOperand, 0=RnRegisterOperand
//!     A: 1=Subtract,           0=Add
//!     H: 1=RsIsHi,             0=RsIsLo
//!     h: 1=RdIsHi,             0=RdIsLo
//!     L: 1=Load,               0=Store
//!     B: 1=TransferBytes,      0=TransferWords
//!     b: 1=TransferBytes,      0=TransferWords
//!     W: 1=LoadHalfword,       0=LoadByteOrStoreHalfword
//!     S: 1=TransferSigned,     0=TransferUnsigned
//!     P: 1=BaseIsSP,           0=BaseIsPC
//!     R: 1=StoreLRloadPC,      0=Dont
//!     X: 1=LowBL,              0=HighBL
//! ```
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use std::mem;
use super::armcondition::*;
use super::super::error::*;
use super::arm7tdmi::exec::armdpop::*;
use super::arm7tdmi::exec::armbsop::*;

pub use self::display::*;

pub mod display;


/// A decoded THUMB high register operation.
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum HiRegisterOp {
    #[doc = "ADD without modifying CPSR flags."]    AddNoFlags = 0,
    #[doc = "CMP which always updates CPSR flags."] CmpFlags   = 1,
    #[doc = "MOV without modifying CPSR flags."]    MovNoFlags = 2,
    #[doc = "BX by register Rs/Hs."]                BxRsHs     = 3,
}

/// A decoded THUMB halfword and signed data transfer operation.
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum LdrhStrhOp {
    #[doc = "Store halfword."]         STRH = 0,
    #[doc = "Load unsigned halfword."] LDRH = 1,
    #[doc = "Load signed byte."]       LDSB = 2,
    #[doc = "Load signed halfword."]   LDSH = 3,
}


/// A decoded THUMB opcode.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ThumbOpcode {
    #[doc = "ADD/SUB with an immediate value or register."] AddSub,
    #[doc = "Shifted register transfer."]                   MoveShiftedReg,
    #[doc = "ALU operations updating CPSR flags."]          DataProcessingFlags,
    #[doc = "MUL with updating CPSR flags."]                AluMul,
    #[doc = "ALU Operations."]                              AluOperation,
    #[doc = "ALU operations with high registers or BX."]    HiRegOpBx,
    #[doc = "LDR relative to PC."]                          LdrPcImm,
    #[doc = "LDR/STR with register offset."]                LdrStrReg,
    #[doc = "LDRH/STRH with register offset."]              LdrhStrhReg,
    #[doc = "LDR/STR with immediate offset."]               LdrStrImm,
    #[doc = "LDRH/STRH with immediate offset."]             LdrhStrhImm,
    #[doc = "LDR/STR relative to SP."]                      LdrStrSpImm,
    #[doc = "Calculate an address relative to PC/SP."]      CalcAddrImm,
    #[doc = "Add an offset to SP."]                         AddSpOffs,
    #[doc = "Push/pop registers onto/from the stack."]      PushPopRegs,
    #[doc = "LDM/STM instruction."]                         LdmStmRegs,
    #[doc = "Causes a software interrupt."]                 SoftwareInterrupt,
    #[doc = "Conditional branch relative to PC."]           BranchConditionOffs,
    #[doc = "Unconditional branch relative to PC."]         BranchOffs,
    #[doc = "Unconditional branch using a 23-bit offset."]  BranchLongOffs,
}


/// A decoded THUMB instruction providing lots
/// of utility and decoding functions to ease
/// THUMB instruction emulation.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ThumbInstruction {
    raw: u16,
    op: ThumbOpcode,
}

impl ThumbInstruction {
    /// A 16-bit pseudo NOP instruction.
    pub const NOP_RAW: u16 = 0b010001_10_11_000_000_u16;
    //                         MOV           R8, R8 (No CPSR update.)

    /// Creates a pseudo NOP instruction.
    ///
    /// The generated instruction moves R8 into R8
    /// without updating any of the CPSR flags.
    pub fn nop() -> ThumbInstruction {
        ThumbInstruction { raw: ThumbInstruction::NOP_RAW, op: ThumbOpcode::HiRegOpBx }
    }

    /// Decodes a raw 16-bit integer as a THUMB instruction.
    #[cfg_attr(feature="clippy", allow(if_same_then_else))] // Order of checks matters a lot here, false positive.
    pub fn decode(raw: u16) -> Result<ThumbInstruction, GbaError> {
        // Decode the opcode to something easier to compare and match.
        let op: ThumbOpcode =
             if (raw & 0xF800) == 0x1800 { ThumbOpcode::AddSub }
        else if (raw & 0xE000) == 0x0000 { ThumbOpcode::MoveShiftedReg }
        else if (raw & 0xE000) == 0x2000 { ThumbOpcode::DataProcessingFlags }
        else if (raw & 0xFFC0) == 0x4340 { ThumbOpcode::AluMul } // Decoded separately as MUL is no ARM state data processing.
        else if (raw & 0xFC00) == 0x4000 { ThumbOpcode::AluOperation }
        else if (raw & 0xFC00) == 0x4400 { ThumbOpcode::HiRegOpBx }
        else if (raw & 0xF800) == 0x4800 { ThumbOpcode::LdrPcImm }
        else if (raw & 0xF200) == 0x5000 { ThumbOpcode::LdrStrReg }
        else if (raw & 0xF200) == 0x5200 { ThumbOpcode::LdrhStrhReg }
        else if (raw & 0xE000) == 0x6000 { ThumbOpcode::LdrStrImm }
        else if (raw & 0xF000) == 0x8000 { ThumbOpcode::LdrhStrhImm }
        else if (raw & 0xF000) == 0x9000 { ThumbOpcode::LdrStrSpImm }
        else if (raw & 0xF000) == 0xA000 { ThumbOpcode::CalcAddrImm }
        else if (raw & 0xFF00) == 0xB000 { ThumbOpcode::AddSpOffs }
        else if (raw & 0xF600) == 0xB400 { ThumbOpcode::PushPopRegs }
        else if (raw & 0xF000) == 0xC000 { ThumbOpcode::LdmStmRegs }
        else if (raw & 0xFF00) == 0xDF00 { ThumbOpcode::SoftwareInterrupt }
        else if (raw & 0xFF00) == 0xDE00 { return Err(GbaError::InvalidThumbInstruction(raw)); } // BAL is undefined.
        else if (raw & 0xF000) == 0xD000 { ThumbOpcode::BranchConditionOffs }
        else if (raw & 0xF800) == 0xE000 { ThumbOpcode::BranchOffs }
        else if (raw & 0xF000) == 0xF000 { ThumbOpcode::BranchLongOffs }
        else { return Err(GbaError::InvalidThumbInstruction(raw)); };

        // Done decoding!
        Ok(ThumbInstruction { raw: raw, op: op })
    }

    /// Decodes the register operand index `Rd`.
    #[allow(non_snake_case)]
    pub fn Rd(&self) -> usize { ((self.raw     ) & 0b111) as usize }

    /// Decodes the register operand index `Rs`.
    #[allow(non_snake_case)]
    pub fn Rs(&self) -> usize { ((self.raw >> 3) & 0b111) as usize }

    /// Decodes the register operand index `Rn`.
    #[allow(non_snake_case)]
    pub fn Rn(&self) -> usize { ((self.raw >> 6) & 0b111) as usize }

    /// Decodes the register operand index `Rm`.
    #[allow(non_snake_case)]
    pub fn Rm(&self) -> usize { ((self.raw >> 8) & 0b111) as usize } // Yes, 8, not 9!

    /// Decodes the register operand index `Hd`.
    ///
    /// `Hd` is a 4-bit register index, whereas
    /// `Rd` only is a 3-bit index. `Hd` is
    /// generated by adding the operand's "High"
    /// bit to the 3-bit index.
    #[allow(non_snake_case)]
    pub fn Hd(&self) -> usize { self.Rd() | (((self.raw >> 4) & 0b1000) as usize) } // Bit 6 = Hd/Rd

    /// Decodes the register operand index `Hs`.
    ///
    /// `Hs` is a 4-bit register index, whereas
    /// `Rs` only is a 3-bit index. `Hs` is
    /// generated by adding the operand's "High"
    /// bit to the 3-bit index.
    #[allow(non_snake_case)]
    pub fn Hs(&self) -> usize { self.Rs() | (((self.raw >> 3) & 0b1000) as usize) } // Bit 7 = Hs/Rs

    /// Determines by how many bits a register value should be shifted.
    pub fn shift_operand(&self) -> u32 { ((self.raw >> 6) & 0b1_1111) as u32 }

    /// Extracts a 5-bit positive immediate value.
    pub fn imm5(&self) -> i32 { ((self.raw >> 6) & 0b1_1111) as i32 }

    /// Extracts a 6-bit positive immediate value.
    pub fn imm6(&self) -> i32 { self.imm5() << 1 }

    /// Extracts a 7-bit positive immediate value.
    pub fn imm7(&self) -> i32 { self.imm5() << 2 }

    /// Extracts an 8-bit positive immediate value.
    pub fn imm8(&self) -> i32 { (self.raw & 0xFF) as i32 }

    /// Extracts a 10-bit positive immediate value.
    pub fn imm10(&self) -> i32 { self.imm8() << 2 }

    /// Extracts a 9-bit signed offset value.
    pub fn offs9(&self) -> i32 { ((((self.raw & 0xFF) as u32) << 24) as i32) >> 23 }

    /// Extracts a 12-bit signed offset value.
    pub fn offs12(&self) -> i32 { ((((self.raw & 0x7FF) as u32) << 21) as i32) >> 20 }

    /// Extracts a raw 11-bit number for long 23-bit offset branches.
    pub fn long_offs_part(&self) -> i32 { (self.raw & 0x7FF) as i32 }

    /// Extracts the comment field of a SWI instruction.
    pub fn comment(&self) -> u8 { (self.raw & 0xFF) as u8 }

    /// Extracts the register list of an LDM/STM instruction.
    pub fn register_list(&self) -> u8 { self.comment() }

    /// Extracts a data processing opcode for the `AddSub` instruction.
    #[allow(non_snake_case)]
    pub fn dpop_AddSub(&self) -> ArmDPOP { if 0 == (self.raw & (1 << 9)) { ArmDPOP::ADD } else { ArmDPOP::SUB } }

    /// Extracts a barrel shifter opcode for the `MoveShiftedReg` instruction.
    #[allow(non_snake_case)]
    pub fn bsop_MoveShiftedReg(&self) -> ArmBSOP { ArmBSOP::decode_immediate((self.raw >> 11) as u32, self.imm5() as u32) }

    /// Extracts a data processing opcode for the `DataProcessingFlags` instruction.
    #[allow(non_snake_case)]
    pub fn dpop_DataProcessingFlags(&self) -> ArmDPOP {
        match (self.raw >> 11) & 0b11 {
            0 => ArmDPOP::MOV,
            1 => ArmDPOP::CMP,
            2 => ArmDPOP::ADD,
            3 => ArmDPOP::SUB,
            _ => unreachable!(),
        }
    }

    /// Extracts data processing and barrel shifter opcodes for the `AluOperation` instruction.
    #[allow(non_snake_case)]
    pub fn dpop_bsop_AluOperation(&self) -> (ArmDPOP, ArmBSOP) {
        match (self.raw >> 6) & 0b1111 {
             0 => (ArmDPOP::AND, ArmBSOP::NOP),
             1 => (ArmDPOP::EOR, ArmBSOP::NOP),
             2 => (ArmDPOP::MOV, ArmBSOP::LSL_Reg(self.Rs())),
             3 => (ArmDPOP::MOV, ArmBSOP::LSR_Reg(self.Rs())),
             4 => (ArmDPOP::MOV, ArmBSOP::ASR_Reg(self.Rs())),
             5 => (ArmDPOP::ADC, ArmBSOP::NOP),
             6 => (ArmDPOP::SBC, ArmBSOP::NOP),
             7 => (ArmDPOP::MOV, ArmBSOP::ROR_Reg(self.Rs())),
             8 => (ArmDPOP::TST, ArmBSOP::NOP),
             9 => (ArmDPOP::RSB, ArmBSOP::NOP), // FIXME NEG a, b => RSB a, b, #0 (Where does 0 come from?!)
            10 => (ArmDPOP::CMP, ArmBSOP::NOP),
            11 => (ArmDPOP::CMN, ArmBSOP::NOP),
            12 => (ArmDPOP::ORR, ArmBSOP::NOP),
            13 => panic!("Decoded AluOperation instead of AluMul!"),
            14 => (ArmDPOP::BIC, ArmBSOP::NOP),
            15 => (ArmDPOP::MVN, ArmBSOP::NOP),
            _ => unreachable!(),
        }
    }

    /// Extracts a high register operation code for the `HiRegOpBx` instruction.
    #[allow(non_snake_case)]
    pub fn op_HiRegOpBx(&self) -> HiRegisterOp { unsafe { mem::transmute(((self.raw >> 8) & 0b11) as u8) } }

    /// Extracts an opcode for the `LdrhStrhReg` instruction.
    #[allow(non_snake_case)]
    pub fn op_LdrhStrhReg(&self) -> LdrhStrhOp { unsafe { mem::transmute(((self.raw >> 10) & 0b11) as u8) } }

    /// Extracts the condition code of a branch instruction.
    pub fn condition(&self) -> ArmCondition { unsafe { mem::transmute(((self.raw >> 8) & 0x0F) as u8) } }

    /// To be used with the `AddSub` opcode.
    #[allow(non_snake_case)]
    pub fn is_Rn_immediate(&self) -> bool { 0 != (self.raw & (1 << 10)) }

    /// Checks whether the given instruction is a load or store instruction.
    pub fn is_load(&self) -> bool { 0 != (self.raw & (1 << 11)) }

    /// Checks whether the given load/store instruction transfers a single byte.
    pub fn is_transfering_bytes(&self) -> bool { 0 != (self.raw & (1 << 10)) }

    /// Checks whether this load/store instruction transfers signed data.
    pub fn is_signed(&self) -> bool { self.is_transfering_bytes() }

    /// Checks whether an address calculation instruction uses SP or PC.
    #[allow(non_snake_case)]
    pub fn is_base_SP(&self) -> bool { self.is_load() }

    /// Checks whether a PUSH/POP instruction does what this function's name implies.
    #[allow(non_snake_case)]
    pub fn is_storing_LR_loading_PC(&self) -> bool { 0 != (self.raw & (1 << 8)) }

    /// Checks whether a 23-bit offset branch loads the higher offset half and jumps.
    pub fn is_low_offset_and_branch(&self) -> bool { self.is_load() }
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
