// License below.
//! Implements THUMB state disassembly.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use std::fmt;
use super::*;
use super::super::arm7tdmi::{Arm7Tdmi, ArmDPOP, ArmBSOP};

impl fmt::Display for ThumbInstruction {
    /// Writes a disassembly of the given instruction to a formatter.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{:06X}\t", self.raw));

        match self.op {
            ThumbOpcode::AddSub              => self.fmt_AddSub(f),
            ThumbOpcode::MoveShiftedReg      => self.fmt_MoveShiftedReg(f),
            ThumbOpcode::DataProcessingFlags => self.fmt_DataProcessingFlags(f),
            ThumbOpcode::AluMul              => self.fmt_AluMul(f),
            ThumbOpcode::AluOperation        => self.fmt_AluOperation(f),
            ThumbOpcode::HiRegOpBx           => self.fmt_HiRegOpBx(f),
            ThumbOpcode::LdrPcImm            => self.fmt_LdrPcImm(f),
            ThumbOpcode::LdrStrReg           => self.fmt_LdrStrReg(f),
            ThumbOpcode::LdrhStrhReg         => self.fmt_LdrhStrhReg(f),
            ThumbOpcode::LdrStrImm           => self.fmt_LdrStrImm(f),
            ThumbOpcode::LdrhStrhImm         => self.fmt_LdrhStrhImm(f),
            ThumbOpcode::LdrStrSpImm         => self.fmt_LdrStrSpImm(f),
            ThumbOpcode::CalcAddrImm         => self.fmt_CalcAddrImm(f),

            _ => unimplemented!()
        }
    }
}

impl ThumbInstruction {
    #[allow(non_snake_case)]
    fn fmt_AddSub(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rd = Arm7Tdmi::register_name(self.Rd());
        let rs = Arm7Tdmi::register_name(self.Rs());
        let op = self.dpop_AddSub();
        try!(write!(f, "{}\t{}, {}, ", op, rd, rs));
        if self.is_Rn_immediate() { write!(f, "#{}", self.Rn() as i32) }
        else                      { write!(f,  "{}", Arm7Tdmi::register_name(self.Rn())) }
    }

    #[allow(non_snake_case)]
    fn fmt_MoveShiftedReg(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rd = Arm7Tdmi::register_name(self.Rd());
        let rs = Arm7Tdmi::register_name(self.Rs());
        let (op, x) = match self.bsop_MoveShiftedReg() {
            ArmBSOP::LSL_Imm(x) => ("lsl", x),
            ArmBSOP::LSR_Imm(x) => ("lsr", x),
            ArmBSOP::ASR_Imm(x) => ("asr", x),
            _ => unreachable!(),
        };
        write!(f, "{}\t{}, {}, #{}", op, rd, rs, x)
    }

    #[allow(non_snake_case)]
    fn fmt_DataProcessingFlags(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rd  = Arm7Tdmi::register_name(self.Rd());
        let imm = self.imm8();
        let op  = self.dpop_DataProcessingFlags();
        write!(f, "{}\t{}, #{}", op, rd, imm)
    }

    #[allow(non_snake_case)]
    fn fmt_AluMul(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "mul\t{}, {}", Arm7Tdmi::register_name(self.Rd()), Arm7Tdmi::register_name(self.Rs()))
    }

    #[allow(non_snake_case)]
    fn fmt_AluOperation(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (dpop, bsop) = self.dpop_bsop_AluOperation();
        let rd = Arm7Tdmi::register_name(self.Rd());
        let rs = Arm7Tdmi::register_name(self.Rs());

        if dpop == ArmDPOP::MOV { write!(f, "{}\t{}, {}", bsop.name(), rd, rs) }
        else                    { write!(f, "{}\t{}, {}", dpop, rd, rs) }
    }

    #[allow(non_snake_case)]
    fn fmt_HiRegOpBx(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let hd = Arm7Tdmi::register_name(self.Hd());
        let hs = Arm7Tdmi::register_name(self.Hs());
        match self.op_HiRegOpBx() {
            HiRegisterOp::AddNoFlags => write!(f, "add\t{}, {}", hd, hs),
            HiRegisterOp::CmpFlags   => write!(f, "cmp\t{}, {}", hd, hs),
            HiRegisterOp::MovNoFlags => write!(f, "mov\t{}, {}", hd, hs),
            HiRegisterOp::BxRsHs     => write!(f, "bx\t{}", hs),
        }
    }

    #[allow(non_snake_case)]
    fn fmt_LdrPcImm(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ldr\t{}, [PC, #{}]", Arm7Tdmi::register_name(self.Rm()), self.imm10())
    }

    #[allow(non_snake_case)]
    fn fmt_LdrStrReg(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}\t{}, [{}, {}]",
            if self.is_load() { "ldr" } else { "str" },
            if self.is_transfering_bytes() { 'b' } else { ' ' },
            Arm7Tdmi::register_name(self.Rd()),
            Arm7Tdmi::register_name(self.Rs()),
            Arm7Tdmi::register_name(self.Rn()),
        )
    }

    #[allow(non_snake_case)]
    fn fmt_LdrhStrhReg(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rd = Arm7Tdmi::register_name(self.Rd());
        let rs = Arm7Tdmi::register_name(self.Rs());
        let rn = Arm7Tdmi::register_name(self.Rn());
        let op = match self.op_LdrhStrhReg() {
            LdrhStrhOp::STRH => "strh",
            LdrhStrhOp::LDRH => "ldrh",
            LdrhStrhOp::LDSB => "ldsb",
            LdrhStrhOp::LDSH => "ldsh",
        };
        write!(f, "{}\t{}, [{}, {}]", op, rd, rs, rn)
    }

    #[allow(non_snake_case)]
    fn fmt_LdrStrImm(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}\t{}, [{}, #{}]",
            if self.is_load() { "ldr" } else { "str" },
            if self.is_transfering_bytes() { 'b' } else { ' ' },
            Arm7Tdmi::register_name(self.Rd()),
            Arm7Tdmi::register_name(self.Rs()),
            if self.is_transfering_bytes() { self.imm5() } else { self.imm7() },
        )
    }

    #[allow(non_snake_case)]
    fn fmt_LdrhStrhImm(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}, [{}, #{}]",
            if self.is_load() { "ldrh" } else { "strh" },
            Arm7Tdmi::register_name(self.Rd()),
            Arm7Tdmi::register_name(self.Rs()),
            self.imm6(),
        )
    }

    #[allow(non_snake_case)]
    fn fmt_LdrStrSpImm(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}, [SP, #{}]",
            if self.is_load() { "ldr" } else { "str" },
            Arm7Tdmi::register_name(self.Rd()),
            self.imm10(),
        )
    }

    #[allow(non_snake_case)]
    fn fmt_CalcAddrImm(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "add\t{}, {}, #{}",
            Arm7Tdmi::register_name(self.Rd()),
            if self.is_base_SP() { "SP" } else { "PC" },
            self.imm10(),
        )
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
