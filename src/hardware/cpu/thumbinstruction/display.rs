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
        try!(write!(f, "{:#06X}\t", self.raw));

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
            ThumbOpcode::AddSpOffs           => self.fmt_AddSpOffs(f),
            ThumbOpcode::PushPopRegs         => self.fmt_PushPopRegs(f),
            ThumbOpcode::LdmStmRegs          => self.fmt_LdmStmRegs(f),
            ThumbOpcode::SoftwareInterrupt   => self.fmt_SoftwareInterrupt(f),
            ThumbOpcode::BranchConditionOffs => self.fmt_BranchConditionOffs(f),
            ThumbOpcode::BranchOffs          => self.fmt_BranchOffs(f),
            ThumbOpcode::BranchLongOffs      => self.fmt_BranchLongOffs(f),
        }
    }
}

impl ThumbInstruction {
    #[allow(non_snake_case)]
    fn fmt_AddSub(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rd = Arm7Tdmi::register_name(self.Rd());
        let rs = Arm7Tdmi::register_name(self.Rs());
        let op = self.dpop_AddSub();
        try!(write!(f, "{}s\t{}, {}, ", op, rd, rs));
        if self.is_Rn_immediate() { write!(f, "#{}", self.Rn() as i32) }
        else                      { write!(f,  "{}", Arm7Tdmi::register_name(self.Rn())) }
    }

    #[allow(non_snake_case)]
    fn fmt_MoveShiftedReg(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rd = Arm7Tdmi::register_name(self.Rd());
        let rs = Arm7Tdmi::register_name(self.Rs());
        let (op, x) = match self.bsop_MoveShiftedReg() {
            ArmBSOP::NOP => { return write!(f, "lsls\t{}, {}, #0", rd, rs); }
            ArmBSOP::LSL_Imm(x) => ("lsls", x),
            ArmBSOP::LSR_Imm(x) => ("lsrs", x),
            ArmBSOP::ASR_Imm(x) => ("asrs", x),
            _ => unreachable!()
        };
        write!(f, "{}\t{}, {}, #{}", op, rd, rs, x)
    }

    #[allow(non_snake_case)]
    fn fmt_DataProcessingFlags(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rd  = Arm7Tdmi::register_name(self.Rd());
        let imm = self.imm8();
        let op  = self.dpop_DataProcessingFlags();
        let s   = if op == ArmDPOP::CMP { ' ' } else { 's' };
        write!(f, "{}{}\t{}, #{}", op, s, rd, imm)
    }

    #[allow(non_snake_case)]
    fn fmt_AluMul(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "muls\t{}, {}", Arm7Tdmi::register_name(self.Rd()), Arm7Tdmi::register_name(self.Rs()))
    }

    #[allow(non_snake_case)]
    fn fmt_AluOperation(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (dpop, bsop) = self.dpop_bsop_AluOperation();
        let rd = Arm7Tdmi::register_name(self.Rd());
        let rs = Arm7Tdmi::register_name(self.Rs());

        if dpop == ArmDPOP::MOV { write!(f, "{}s\t{}, {}", bsop.name(), rd, rs) }
        else { write!(f, "{}{}\t{}, {}", dpop, if dpop.is_test() { ' ' } else { 's' }, rd, rs) }
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

    #[allow(non_snake_case)]
    fn fmt_AddSpOffs(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "add\tSP, #{}", self.offs9()) }

    #[allow(non_snake_case)]
    fn fmt_PushPopRegs(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{}\t", if self.is_load() { "pop" } else { "push" }));
        let with_reg = if self.is_storing_LR_loading_PC() {
            if self.is_load() { Some(15_usize) } else { Some(14_usize) }
        } else { None };
        self.fmt_register_list(f, with_reg)
    }

    #[allow(non_snake_case)]
    fn fmt_LdmStmRegs(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{}\t{}!, ",
            if self.is_load() { "ldmia" } else { "stmia" },
            Arm7Tdmi::register_name(self.Rm()),
        ));
        self.fmt_register_list(f, None)
    }

    fn fmt_register_list(&self, f: &mut fmt::Formatter, with_reg: Option<usize>) -> fmt::Result {
        try!(write!(f, "{{"));
        let regs = self.register_list();
        let mut any = false;
        for i in 0..8 { if 0 != (regs & (1 << i)) {
            if any { try!(write!(f, ", ")); }
            any = true;
            try!(write!(f, "{}", Arm7Tdmi::register_name(i as usize)));
        }}
        if let Some(r) = with_reg {
            if any { try!(write!(f, ", ")); }
            try!(write!(f, "{}", Arm7Tdmi::register_name(r)));
        }
        write!(f, "}}")
    }

    #[allow(non_snake_case)]
    fn fmt_SoftwareInterrupt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "swi\t{}", self.comment()) }

    #[allow(non_snake_case)]
    fn fmt_BranchConditionOffs(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "b{}\t{}", self.condition().assembly_name(), self.offs9())
    }

    #[allow(non_snake_case)]
    fn fmt_BranchOffs(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "b\t#{}", self.offs12()) }

    #[allow(non_snake_case)]
    fn fmt_BranchLongOffs(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let offs = self.long_offs_part();
        if self.is_low_offset_and_branch() { write!(f, "bl1\t#{:010X}",  (offs <<  1) as u32) }
        else                               { write!(f, "bl0\t#{:010X}", ((offs << 21) >> 10) as u32) }
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
