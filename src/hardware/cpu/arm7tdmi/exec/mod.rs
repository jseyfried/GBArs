// License below.
//! Implements common execution logic for THUMB and ARM state instruction.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use super::*;

pub use self::armdpop::*;
pub use self::armbsop::*;
pub use self::execarm::*;
pub use self::armcondition::*;

pub mod armdpop;
pub mod armbsop;
pub mod armcondition;
pub mod execarm;

impl Arm7Tdmi {
    fn alu_data_processing(&self, dpop: ArmDPOP, op1: i32, op2: i32) -> i32 {
        let c = self.cpsr.C() as i32;
        match dpop {
            ArmDPOP::AND => { op1 & op2 },
            ArmDPOP::EOR => { op1 ^ op2 },
            ArmDPOP::SUB => { op1.wrapping_sub(op2) },
            ArmDPOP::RSB => { op2.wrapping_sub(op1) },
            ArmDPOP::ADD => { op1.wrapping_add(op2) },
            ArmDPOP::ADC => { op1.wrapping_add(op2).wrapping_add(c) },
            ArmDPOP::SBC => { op1.wrapping_sub(op2).wrapping_sub(1-c) },
            ArmDPOP::RSC => { op2.wrapping_sub(op1).wrapping_sub(1-c) },
            ArmDPOP::TST => panic!("TST that should be MSR/MRS!"),
            ArmDPOP::TEQ => panic!("TEQ that should be MSR/MRS!"),
            ArmDPOP::CMP => panic!("CMP that should be MSR/MRS!"),
            ArmDPOP::CMN => panic!("CMN that should be MSR/MRS!"),
            ArmDPOP::ORR => { op1 | op2 },
            ArmDPOP::MOV => { op2 },
            ArmDPOP::BIC => { op1 & !op2 },
            ArmDPOP::MVN => { !op2 },
        }
    }

    fn alu_data_processing_flags(&mut self, dpop: ArmDPOP, op1: i32, op2: i32, shift_carry: bool) -> Option<i32> {
        let c = self.cpsr.C() as i32;
        let mut cf = shift_carry;
        let mut vf = self.cpsr.V(); // Emulate "not touching V" by writing back "old V".

        let res: i32 = match dpop {
            ArmDPOP::AND | ArmDPOP::TST => { op1 & op2 },
            ArmDPOP::EOR | ArmDPOP::TEQ => { op1 ^ op2 },
            ArmDPOP::SUB | ArmDPOP::CMP => { Arm7Tdmi::alu_sub_carry_overflow(op1, op2, &mut cf, &mut vf) },
            ArmDPOP::RSB                => { Arm7Tdmi::alu_sub_carry_overflow(op2, op1, &mut cf, &mut vf) },
            ArmDPOP::ADD | ArmDPOP::CMN => { Arm7Tdmi::alu_add_carry_overflow(op1, op2, &mut cf, &mut vf) },
            ArmDPOP::ADC                => { Arm7Tdmi::alu_add_carry_overflow(op1, op2.wrapping_add(c), &mut cf, &mut vf) },
            ArmDPOP::SBC                => { Arm7Tdmi::alu_sub_carry_overflow(op1, op2.wrapping_sub(1-c), &mut cf, &mut vf) },
            ArmDPOP::RSC                => { Arm7Tdmi::alu_sub_carry_overflow(op2, op1.wrapping_sub(1-c), &mut cf, &mut vf) },
            ArmDPOP::ORR                => { op1 | op2 },
            ArmDPOP::MOV                => { op2 },
            ArmDPOP::BIC                => { op1 & !op2 },
            ArmDPOP::MVN                => { !op2 },
        };

        // PSR transfer will override this in the caller.
        self.cpsr.set_N(res < 0);
        self.cpsr.set_Z(res == 0);
        self.cpsr.set_C(cf);
        self.cpsr.set_V(vf);

        if dpop.is_test() { None } else { Some(res) }
    }

    fn alu_add_carry_overflow(a: i32, b: i32, c: &mut bool, v: &mut bool) -> i32 {
        let res64: u64 = (a as u32 as u64).wrapping_add(b as u32 as u64);
        *c = 0 != (res64 & (1 << 32));
        let x = a.overflowing_add(b);
        *v = x.1;
        x.0
    }

    fn alu_sub_carry_overflow(a: i32, b: i32, c: &mut bool, v: &mut bool) -> i32 {
        let res64: u64 = (a as u32 as u64).wrapping_sub(b as u32 as u64);
        *c = 0 != (res64 & (1 << 32));
        let x = a.overflowing_sub(b);
        *v = x.1;
        x.0
    }


    fn alu_barrel_shifter(&mut self, bsop: ArmBSOP, op1: i32) -> i32 {
        match bsop {
            ArmBSOP::LSL_Imm(x) => op1 << x,
            ArmBSOP::LSR_Imm(x) => ((op1 as u32) >> x) as i32,
            ArmBSOP::ASR_Imm(x) => op1 >> x,
            ArmBSOP::ROR_Imm(x) => op1.rotate_right(x),
            ArmBSOP::NOP        => op1,
            ArmBSOP::LSR_32     => 0,
            ArmBSOP::ASR_32     => op1 >> 31,
            ArmBSOP::RRX        => Arm7Tdmi::alu_rrx(op1, self.cpsr.C()),
            ArmBSOP::LSL_Reg(r) => Arm7Tdmi::alu_lsl_reg(op1, (self.gpr[r] as u32) & 0xFF),
            ArmBSOP::LSR_Reg(r) => Arm7Tdmi::alu_lsr_reg(op1, (self.gpr[r] as u32) & 0xFF),
            ArmBSOP::ASR_Reg(r) => Arm7Tdmi::alu_asr_reg(op1, (self.gpr[r] as u32) & 0xFF),
            ArmBSOP::ROR_Reg(r) => Arm7Tdmi::alu_ror_reg(op1, (self.gpr[r] as u32) & 0xFF),
        }
    }

    fn alu_barrel_shifter_carry(&mut self, bsop: ArmBSOP, op1: i32) -> (i32, bool) {
        match bsop {
            ArmBSOP::LSL_Imm(x) => Arm7Tdmi::alu_lsl_imm_carry(op1, x),
            ArmBSOP::LSR_Imm(x) => Arm7Tdmi::alu_lsr_imm_carry(op1, x),
            ArmBSOP::ASR_Imm(x) => Arm7Tdmi::alu_asr_imm_carry(op1, x),
            ArmBSOP::ROR_Imm(x) => Arm7Tdmi::alu_ror_imm_carry(op1, x),
            ArmBSOP::NOP        => (op1, self.cpsr.C()),
            ArmBSOP::LSR_32     => (((op1 as u32) >> 31) as i32, false),
            ArmBSOP::ASR_32     => (op1 >> 31, 0 != (op1 & (1 << 31))),
            ArmBSOP::RRX        => (Arm7Tdmi::alu_rrx(op1, self.cpsr.C()), 0 != (op1 & 0b1)),
            ArmBSOP::LSL_Reg(r) => Arm7Tdmi::alu_lsl_reg_carry(op1, (self.gpr[r] as u32) & 0xFF, self.cpsr.C()),
            ArmBSOP::LSR_Reg(r) => Arm7Tdmi::alu_lsr_reg_carry(op1, (self.gpr[r] as u32) & 0xFF, self.cpsr.C()),
            ArmBSOP::ASR_Reg(r) => Arm7Tdmi::alu_asr_reg_carry(op1, (self.gpr[r] as u32) & 0xFF, self.cpsr.C()),
            ArmBSOP::ROR_Reg(r) => Arm7Tdmi::alu_ror_reg_carry(op1, (self.gpr[r] as u32) & 0xFF, self.cpsr.C()),
        }
    }

    fn alu_rrx(op1: i32, c: bool) -> i32 { ((c as i32) << 31) | (((op1 as u32) >> 1) as i32) }
    fn alu_lsl_imm_carry(op1: i32, op2: u32) -> (i32, bool) { (op1 << op2, 0 != ((op1 >> (32-op2)) & 0b1)) }
    fn alu_lsr_imm_carry(op1: i32, op2: u32) -> (i32, bool) { (((op1 as u32) >> op2) as i32, 0 != ((op1 >> (op2-1)) & 0b1)) }
    fn alu_asr_imm_carry(op1: i32, op2: u32) -> (i32, bool) { (op1 >> op2, 0 != ((op1 >> (op2-1)) & 0b1)) }
    fn alu_ror_imm_carry(op1: i32, op2: u32) -> (i32, bool) { (op1.rotate_right(op2), 0 != ((op1 >> (op2-1)) & 0b1)) }
    fn alu_lsl_reg(op1: i32, op2: u32) -> i32 { if op2 < 32 { op1 << op2 } else { 0 } }
    fn alu_asr_reg(op1: i32, op2: u32) -> i32 { if op2 < 32 { op1 >> op2 } else { op1 >> 31 } }
    fn alu_ror_reg(op1: i32, op2: u32) -> i32 { op1.rotate_right(op2 % 32) }
    fn alu_lsr_reg(op1: i32, op2: u32) -> i32 {
        if op2 < 32 { ((op1 as u32) >> op2) as i32 }
        else { ((op1 as u32) >> 31) as i32 }
    }
    fn alu_lsl_reg_carry(op1: i32, op2: u32, c: bool) -> (i32, bool) { match op2 {
        0           => (op1, c),
        x if x < 32 => Arm7Tdmi::alu_lsl_imm_carry(op1, op2),
        32          => (0, 0 != (op1 & 0b1)),
        _           => (0, false),
    }}
    fn alu_lsr_reg_carry(op1: i32, op2: u32, c: bool) -> (i32, bool) { match op2 {
        0           => (op1, c),
        x if x < 32 => Arm7Tdmi::alu_lsr_imm_carry(op1, op2),
        32          => (0, 0 != (op1 & (1 << 31))),
        _           => (0, false),
    }}
    fn alu_asr_reg_carry(op1: i32, op2: u32, c: bool) -> (i32, bool) { match op2 {
        0           => (op1, c),
        x if x < 32 => Arm7Tdmi::alu_asr_imm_carry(op1, op2),
        _           => (op1 >> 31, 0 != (op1 & (1 << 31))),
    }}
    fn alu_ror_reg_carry(op1: i32, op2: u32, c: bool) -> (i32, bool) { match op2 {
        0  => (op1, c),
        32 => (op1, 0 != (op1 & (1 << 31))),
        x  => Arm7Tdmi::alu_ror_imm_carry(op1, op2 % 32),
    }}
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
