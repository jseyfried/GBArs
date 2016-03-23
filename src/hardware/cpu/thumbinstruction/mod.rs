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

/// A decoded THUMB opcode.
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
    /// ## Returns
    /// The generated instruction moves R8 into R8
    /// without updating any of the CPSR flags.
    pub fn nop() -> ThumbInstruction {
        ThumbInstruction { raw: ThumbInstruction::NOP_RAW, op: ThumbOpcode::HiRegOpBx }
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
