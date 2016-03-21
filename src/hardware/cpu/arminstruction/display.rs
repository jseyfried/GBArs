// License below.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use super::*;
use super::super::armcondition::ArmCondition;
use std::fmt;

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

        match self.op {
            ArmOpcode::Unknown        => write!(f, "<unknown>"),
            ArmOpcode::SWI            => self.fmt_swi(f),
            ArmOpcode::BX             => self.fmt_bx(f),
            ArmOpcode::B_BL           => self.fmt_b_bl(f),
            ArmOpcode::MRS            => self.fmt_mrs(f),
            ArmOpcode::MSR_Reg        => self.fmt_msr_reg(f),
            ArmOpcode::MSR_Flags      => self.fmt_msr_flags(f),
            ArmOpcode::SWP            => self.fmt_swp(f),
            ArmOpcode::CDP            => self.fmt_cdp(f),
            ArmOpcode::MRC_MCR        => self.fmt_mrc_mcr(f),
            ArmOpcode::LDC_STC        => self.fmt_ldc_stc(f),
            ArmOpcode::LDM_STM        => self.fmt_ldm_stm(f),
            ArmOpcode::LDR_STR        => self.fmt_ldr_str(f),
            ArmOpcode::LDRH_STRH_Reg  => self.fmt_ldrh_strh_reg(f),
            ArmOpcode::LDRH_STRH_Imm  => self.fmt_ldrh_strh_imm(f),
            ArmOpcode::MUL_MLA        => self.fmt_mul_mla(f),
            ArmOpcode::MULL_MLAL      => self.fmt_mull_mlal(f),
            ArmOpcode::DataProcessing => self.fmt_data_processing(f),
        }
    }
}

impl ArmCondition {
    const CONDITION_NAMES: &'static [&'static str] = &[
        "eq", "ne", "hs", "lo", "mi", "pl", "vs", "vc",
        "hi", "ls", "ge", "lt", "gt", "le",   "", "nv",
    ];

    fn assembly_name(&self) -> &'static str {
        let i = *self as u8 as usize;
        debug_assert!(i < ArmCondition::CONDITION_NAMES.len());
        ArmCondition::CONDITION_NAMES[i]
    }
}

#[allow(non_snake_case)]
impl ArmInstruction {
    const REGISTER_NAMES: &'static [&'static str] = &[
        "R0", "R1", "R2", "R3", "R4", "R5", "R6", "R7",
        "R8", "R9", "R10", "R11", "R12", "SP", "LR", "PC"
    ];

    fn register_name(i: usize) -> &'static str {
        debug_assert!(i < ArmInstruction::REGISTER_NAMES.len());
        ArmInstruction::REGISTER_NAMES[i]
    }

    fn Rn_name(&self) -> &'static str { ArmInstruction::register_name(self.Rn()) }
    fn Rd_name(&self) -> &'static str { ArmInstruction::register_name(self.Rd()) }
    fn Rs_name(&self) -> &'static str { ArmInstruction::register_name(self.Rs()) }
    fn Rm_name(&self) -> &'static str { ArmInstruction::register_name(self.Rm()) }

    fn condition_name(&self) -> &'static str { self.condition().assembly_name() }
    fn psr_name(&self) -> &'static str { if self.is_accessing_spsr() { "SPSR" } else { "CPSR" } }
    fn ld_st_name(&self) -> &'static str { if self.is_load() { "ld" } else { "st" } }
    fn off_sign_name(&self) -> char { if self.is_offset_added() { '+' } else { '-' } }

    fn ldrh_strh_op_name(&self) -> &'static str {
        match (self.raw >> 5) & 0b11 {
            1 => "h",
            2 => "sb",
            3 => "sh",
            _ => { error!("Opcode 00 is illegal, as it encodes other instructions. (MLA, SWP,...)"); "<?>" },
        }
    }

    fn shift_op_name(&self) -> &'static str {
        match (self.raw >> 5) & 0b11 {
            0 => ", lsl ",
            1 => ", lsr ",
            2 => ", asr ",
            3 => ", ror ",
            _ => unreachable!(),
        }
    }

    // Formatting utility functions below.

    fn string_shifted_Rm(&self) -> String {
        let mut s = String::with_capacity(24);
        s.push_str(self.Rm_name());

        // Ignore LSL(0) and handle RRX.
        if (self.raw & 0x0FF0) == 0    { return s; }
        if (self.raw & 0x0FF0) == 0x60 { s.push_str(", rrx"); return s; }

        // Write shift.
        s.push_str(self.shift_op_name());
        if self.is_register_shift_immediate() { s.push_str(format!("#{}", self.register_shift_immediate()).as_str()); }
        else { s.push_str(self.Rs_name()); }

        s
    }

    #[allow(non_snake_case)]
    fn fmt_Rn_offset(&self, f: &mut fmt::Formatter, offs: String) -> fmt::Result {
        if self.is_pre_indexed() {
            write!(f, "[{}, {}]{}", self.Rn_name(), offs, if self.is_auto_incrementing() { "!" } else { "" })
        } else {
            write!(f, "[{}], {}", self.Rn_name(), offs)
        }
    }

    fn fmt_shft_field(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_shift_field_register() {
            write!(f, "{}", self.string_shifted_Rm())
        }
        else { write!(f, "#{}", self.rotated_immediate()) }
    }

    fn fmt_shsr_field(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_shift_field_register() { write!(f, "{}", self.Rm_name()) }
        else { write!(f, "#{:#010X}", self.rotated_immediate() as u32) }
    }

    fn fmt_offs_field(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_offset_field_immediate() {
            self.fmt_Rn_offset(f, format!("#{:+}", self.offset12()))
        } else {
            self.fmt_Rn_offset(f, format!("{}{}", self.off_sign_name(), self.string_shifted_Rm()))
        }
    }

    fn fmt_register_list(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{{"));
        let mut got_first = false;
        for i in 0 .. 16 {
            if (self.raw & (1 << i)) != 0 {
                if got_first {
                    try!(write!(f, ", {}", ArmInstruction::register_name(i)));
                } else {
                    got_first = true;
                    try!(write!(f, "{}", ArmInstruction::register_name(i)));
                }
            }
        }
        write!(f, "}}{}", if self.is_enforcing_user_mode() { "^" } else { "" })
    }

    fn fmt_swi(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "swi{}\t#{:#08X}", self.condition_name(), self.comment())
    }

    fn fmt_bx(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "bx{}\t{}", self.condition_name(), self.Rm_name())
    }

    fn fmt_b_bl(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{inst}{cond}\t#{offs}",
            inst = if self.is_branch_with_link() { "bl" } else { "b" },
            cond = self.condition_name(),
            offs = self.branch_offset() + 8 // Due to pipelining relative to B/BL, not PC.
        )
    }

    fn fmt_mrs(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "mrs{}\t{}, {}", self.condition_name(), self.Rd_name(), self.psr_name())
    }

    fn fmt_msr_reg(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "msr{}\t{}, {}", self.condition_name(), self.psr_name(), self.Rm_name())
    }

    fn fmt_msr_flags(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "msr{}\t{}_flg, ", self.condition_name(), self.psr_name()));
        self.fmt_shsr_field(f)
    }

    fn fmt_swp(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "swp{bytes}{cond}\t{Rd}, {Rm}, [{Rn}]",
            bytes = if self.is_transfering_bytes() { "b" } else { "" },
            cond  = self.condition_name(),
            Rd = self.Rd_name(), Rm = self.Rm_name(), Rn = self.Rn_name()
        )
    }

    fn fmt_cdp(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "cdp{cond}\tP{cpid}, {cpop}, CR{cd}, CR{cn}, CR{cm}, {info}",
            cond = self.condition_name(),
            cpid = self.cp_id(), cpop = self.cp_opcode4(),
            cd = self.Rd(), cn = self.Rn(), cm = self.Rm(),
            info = self.cp_info()
        )
    }

    fn fmt_mrc_mcr(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{inst}{cond}\tP{cpid}, {cpop}, {Rd}, CR{cn}, CR{cm}, {info}",
            inst = if self.is_load() { "mrc" } else { "mcr" },
            cond = self.condition_name(),
            cpid = self.cp_id(), cpop = self.cp_opcode3(),
            Rd = self.Rd_name(), cn = self.Rn(), cm = self.Rm(),
            info = self.cp_info()
        )
    }

    fn fmt_ldc_stc(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{inst}c{blk}{cond}\tP{cpid}, CR{cd}, ",
            inst = self.ld_st_name(),
            blk  = if self.is_register_block_transfer() { "l" } else { "" },
            cond = self.condition_name(), cpid = self.cp_id(), cd = self.Rd()
        ));
        self.fmt_Rn_offset(f, format!("#{:+}", self.offset8()))
    }

    fn fmt_ldm_stm(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{inst}m{inc_dec}{pre}{cond}\t{Rn}{auto}, ",
            inst    = self.ld_st_name(),
            inc_dec = if self.is_offset_added() { 'i' } else { 'd' },
            pre     = if self.is_pre_indexed()  { 'b' } else { 'a' },
            cond    = self.condition_name(), Rn = self.Rn_name(),
            auto    = if self.is_auto_incrementing() { "!" } else { "" }
        ));
        self.fmt_register_list(f)
    }

    fn fmt_ldr_str(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{inst}r{bytes}{t}{cond}\t{Rd}, ",
            inst  = self.ld_st_name(),
            bytes = if self.is_transfering_bytes() { "b" } else { "" },
            t     = if !self.is_pre_indexed() & self.is_auto_incrementing() { "t" } else { "" },
            cond  = self.condition_name(), Rd = self.Rd_name()
        ));
        self.fmt_offs_field(f)
    }

    fn fmt_ldrh_strh_common(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}r{}{}\t{}, ", self.ld_st_name(), self.ldrh_strh_op_name(), self.condition_name(), self.Rd_name())
    }

    fn fmt_ldrh_strh_reg(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(self.fmt_ldrh_strh_common(f));
        self.fmt_Rn_offset(f, format!("{}{}", self.off_sign_name(), self.Rm_name()))
    }

    fn fmt_ldrh_strh_imm(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(self.fmt_ldrh_strh_common(f));
        self.fmt_Rn_offset(f, format!("#{:+}", self.split_offset8()))
    }

    fn fmt_mul_mla(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{inst}{flgs}{cond}\t{Rn}, {Rm}, {Rs}",
            inst = if self.is_accumulating() { "mla" } else { "mul" },
            flgs = if self.is_setting_flags() { "s" } else { "" },
            cond = self.condition_name(),
            Rn = self.Rn_name(), Rm = self.Rm_name(), Rs = self.Rs_name()
        ));
        if self.is_accumulating() { write!(f, ", {}", self.Rd_name()) } else { Ok(()) }
    }

    fn fmt_mull_mlal(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{sign}{inst}{flgs}{cond}\t{Rd}, {Rn}, {Rm}, {Rs}",
            sign = if self.is_signed() { 's' } else { 'u' },
            inst = if self.is_accumulating() { "mlal" } else { "mull" },
            flgs = if self.is_setting_flags() { "s" } else { "" },
            cond = self.condition_name(), Rd = self.Rd_name(),
            Rn = self.Rn_name(), Rm = self.Rm_name(), Rs = self.Rs_name()
        )
    }

    fn fmt_data_processing(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let op = self.dpop();
        try!(write!(f, "{dpop}{flgs}{cond}\t",
            dpop = op,
            flgs = if self.is_setting_flags() & !op.is_test() { "s" } else { "" },
            cond = self.condition_name()
        ));
        if !op.is_test() { try!(write!(f, "{}, ", self.Rd_name())); }
        if !op.is_move() { try!(write!(f, "{}, ", self.Rn_name())); }
        self.fmt_shft_field(f)
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
