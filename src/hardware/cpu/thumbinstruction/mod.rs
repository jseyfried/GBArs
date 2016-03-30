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
//!     000o pxxx  xx<S ><D> | Move Shift, Rd = Rs SHIFT(op) xxxxx
//!     001o p<M>  imm_ imm_ | Data Processing, Rm = Rm OP(op) imm_imm_
//!     0100 00_o  p_<S ><D> | ALU Operation, Rd = Rd OP(_op_) Rs
//!     0100 01op  hH<S ><D> | Hi-Reg Op / BX, Rd/Hd = Rd/Hd OP(op) Rs/Hs
//!     0100 1<M>  imm_ imm_ | LDR Rm, [PC, #imm_imm_00]
//!     0101 LB0<  N><S ><D> | LDR/STR Rd, [Rs, Rn]
//!     0101 WS1<  N><S ><D> | LDRH/STRH Rd, [Rs, Rn]
//!     011b L_im  m_<S ><D> | LDR/STR Rd, [Rs, _imm_]
//!     1000 L_im  m_<S ><D> | LDRH/STRH Rd, [Rs, _imm_]
//!     1001 L<M>  imm_ imm_ | LDR/STR Rm, [SP, imm_imm_00]
//!     1010 P<M>  imm_ imm_ | ADD Rm, PC/SP, imm_imm_00
//!     1011 0000  offs offs | ADD SP, SP, SignExtend(offsoffs00)
//!     1011 L10R  regs regs | PUSH/POP regsregs
//!     1100 L<M>  regs regs | LDM/STM Rm, regsregs
//!     1101 1111  comm ent_ | SWI comment_
//!     1101 cond  offs offs | B{cond} SignExtend(offsoffs0)
//!     1110 0off  offs offs | B SignExtend(offoffsoffs0)
//!     1111 Xoff  offs offs | BL Offset23Bit
//!
//! Bit Flags:
//!     I: 1=RnImmediateOperand, 0=RnRegisterOperand
//!     A: 1=Subtract,           0=Add
//!     h: 1=RsIsHi,             0=RsIsLo
//!     H: 1=RdIsHi,             0=RdIsLo
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

use super::super::error::*;


/// A decoded THUMB shift operation.
#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum ShiftOp {
    #[doc = "Logical Shift Left"]     LSL = 0,
    #[doc = "Logical Shift Right"]    LSR = 1,
    #[doc = "Arithmetic Shift Right"] ASR = 2,
}


/// A decoded THUMB opcode.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ThumbOpcode {
    #[doc = "ADD/SUB with an immediate value or register."] AddSub,
    #[doc = "Shifted register transfer."]                   MoveShiftedReg,
    #[doc = "ALU operations updating CPSR flags."]          DataProcessingFlags,
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
    pub fn decode(raw: u16) -> Result<ThumbInstruction, GbaError> {
        // Decode the opcode to something easier to compare and match.
        let op: ThumbOpcode =
             if (raw & 0xF800) == 0x1800 { ThumbOpcode::AddSub }
        else if (raw & 0xE000) == 0x0000 { ThumbOpcode::MoveShiftedReg }
        else if (raw & 0xE000) == 0x2000 { ThumbOpcode::DataProcessingFlags }
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
    pub fn Hd(&self) -> usize { self.Rd() | (((self.raw >> 3) & 0b1000) as usize) } // Bit 6 = Hd/Rd

    /// Decodes the register operand index `Hs`.
    ///
    /// `Hs` is a 4-bit register index, whereas
    /// `Rs` only is a 3-bit index. `Hs` is
    /// generated by adding the operand's "High"
    /// bit to the 3-bit index.
    #[allow(non_snake_case)]
    pub fn Hs(&self) -> usize { self.Rs() | (((self.raw >> 4) & 0b1000) as usize) } // Bit 7 = Hs/Rs

    /// To be used with the `AddSub` opcode.
    #[allow(non_snake_case)]
    pub fn Rn_is_immediate(&self) -> bool { 0 != (self.raw & (1 << 10)) }

    /// Determines by how many bits a register value should be shifted.
    pub fn shift_operand(&self) -> u32 { ((self.raw >> 6) & 0b1_1111) as u32 }
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
