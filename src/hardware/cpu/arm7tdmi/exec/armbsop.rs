// License below.
//! Implements barrel shifter opcodes for the ARM CPU.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use std::fmt;
use super::super::Arm7Tdmi;

/// A barrel shifter opcode.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ArmBSOP {
    #[doc = "No shift (LSL #0)"]                            NOP,
    #[doc = "Logical Shift Left by an immediate value"]     LSL_Imm(u32),
    #[doc = "Logical Shift Right by an immediate value"]    LSR_Imm(u32),
    #[doc = "Arithmetic Shift Right by an immediate value"] ASR_Imm(u32),
    #[doc = "Rotate Right by an immediate value"]           ROR_Imm(u32),
    #[doc = "Rotate Right Extended"]                        RRX,
    #[doc = "Logical Shift Left by a register value"]       LSL_Reg(usize),
    #[doc = "Logical Shift Right by a register value"]      LSR_Reg(usize),
    #[doc = "Arithmetic Shift Right by a register value"]   ASR_Reg(usize),
    #[doc = "Rotate Right by a register value"]             ROR_Reg(usize),
}

impl ArmBSOP {
    /// Decodes a shift opcode from a 2-bit integer.
    pub fn decode_immediate(op: u32, imm: u32) -> ArmBSOP {
        match op & 0b11 {
            0 => if imm == 0 { ArmBSOP::NOP } else { ArmBSOP::LSL_Imm(imm) },
            1 => ArmBSOP::LSR_Imm(imm),
            2 => ArmBSOP::ASR_Imm(imm),
            3 => if imm == 0 { ArmBSOP::RRX } else { ArmBSOP::ROR_Imm(imm) },
            _ => unreachable!(),
        }
    }

    /// Decodes a shift opcode from a 2-bit integer.
    pub fn decode_register(op: u32, reg: usize) -> ArmBSOP {
        match op & 0b11 {
            0 => ArmBSOP::LSL_Reg(reg),
            1 => ArmBSOP::LSR_Reg(reg),
            2 => ArmBSOP::ASR_Reg(reg),
            3 => ArmBSOP::ROR_Reg(reg),
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for ArmBSOP {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ArmBSOP::NOP => Ok(()),
            ArmBSOP::LSL_Imm(x) => write!(f, "lsl #{}", x),
            ArmBSOP::LSR_Imm(x) => write!(f, "lsr #{}", x),
            ArmBSOP::ASR_Imm(x) => write!(f, "asr #{}", x),
            ArmBSOP::ROR_Imm(x) => write!(f, "ror #{}", x),
            ArmBSOP::RRX        => write!(f, "rrx"),
            ArmBSOP::LSL_Reg(x) => write!(f, "lsl {}", Arm7Tdmi::register_name(x)),
            ArmBSOP::LSR_Reg(x) => write!(f, "lsr {}", Arm7Tdmi::register_name(x)),
            ArmBSOP::ASR_Reg(x) => write!(f, "asr {}", Arm7Tdmi::register_name(x)),
            ArmBSOP::ROR_Reg(x) => write!(f, "ror {}", Arm7Tdmi::register_name(x)),
        }
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
