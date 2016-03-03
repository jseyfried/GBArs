// License below.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use super::*;
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
            ArmOpcode::Unknown       => write!(f, "<unknown>"),
            ArmOpcode::SWI           => self.fmt_swi(f),
            ArmOpcode::BX            => self.fmt_bx(f),
            ArmOpcode::B_BL          => self.fmt_b_bl(f),
            ArmOpcode::MRS           => self.fmt_mrs(f),
            ArmOpcode::MSR_Reg       => self.fmt_msr_reg(f),
            ArmOpcode::MSR_Immediate => self.fmt_msr_immediate(f),
            ArmOpcode::SWP           => self.fmt_swp(f),
            ArmOpcode::CDP           => self.fmt_cdp(f),
            ArmOpcode::MRC_MCR       => self.fmt_mrc_mcr(f),
            _ => unimplemented!();
        }
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
        match self.op {
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
                    if !self.is_pre_indexed() & self.is_auto_incrementing() { "t" } else { "" },
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
            ArmOpcode::DataProcessing => {
                let op = self.dpop();
                try!(write!(f, "{}{}{}\t", &op, cond, if self.is_setting_flags() && !op.is_test() { "s" } else { "" }));
                if !op.is_test() { try!(write!(f, "R{}, ", self.Rd())); }
                if !op.is_move() { try!(write!(f, "R{}, ", self.Rn())); }
                self.display_shift(f)
            },
            ArmOpcode::LDC_STC => write!(f, "{}{}{}\tP{}, C{}, [R{}{}, #{}{}{}",
                if self.is_load() { "ldc" } else { "stc" },
                if self.is_register_block_transfer() { "l" } else { "" },
                cond, self.Rs(), self.Rd(), self.Rn(),
                if self.is_pre_indexed() { "" } else { "]" },
                self.offset8(),
                if self.is_pre_indexed() { "]" } else { "" },
                if self.is_auto_incrementing() { "!" } else { "" }
            ),
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

    // Formatting utility functions below.

    fn fmt_msr_shift_field(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_shift_field_register() { write!(f, "{}", self.Rm_name()) }
        else { write!(f, "#{:#010X}", self.rotated_immediate() as u32) }
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
        write!(f, "mrs{cond}\t{Rd}, {PSR}", self.condition_name(), self.Rd_name(), self.psr_name())
    }

    fn fmt_msr_reg(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "msr{cond}\t{PSR}, {Rm}", self.condition_name(), self.psr_name(), self.Rm_name())
    }

    fn fmt_msr_immediate(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "msr{}\t{}_flg, ", self.condition_name(), self.psr_name()));
        self.fmt_msr_shift_field(f)
    }

    fn fmt_swp(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "swp{bytes}{cond}\t{Rd}, {Rm}, [{Rn}]"
            bytes = if self.is_transfering_bytes() { "b" } else { "" },
            cond  = self.condition_name(),
            Rd = self.Rd_name, Rm = self.Rm_name, Rn = self.Rn_name,
        )
    }

    fn fmt_cdp(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "cdp{cond}\tP{cpid}, {cpop}, CR{cd}, CR{cn}, CR{cm}, {info}"
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
