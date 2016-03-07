// License below.
//! Implements emulation utilities for the GBA's main CPU, the ARM7TDMI.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use std::u32;
use super::*;
use super::super::arminstruction::*;
use super::super::super::error::*;

impl Arm7Tdmi {
    /// Immediately executes a single ARM state instruction.
    ///
    /// # Params
    /// - `inst`: The instruction to execute.
    ///
    /// # Returns
    /// - `Ok` if executing the instruction succeeded.
    /// - `Err` if trying to execute an ill-formed instruction.
    #[allow(dead_code)] // TODO delete this
    pub fn execute_arm_state(&mut self, inst: ArmInstruction) -> Result<(), GbaError> {
        let do_exec = try!(inst.condition().check(&self.cpsr));
        if !do_exec { return Ok(()); }

        match inst.opcode() {
            ArmOpcode::BX             => self.execute_bx(inst),
            ArmOpcode::B_BL           => self.execute_b_bl(inst),
            ArmOpcode::MUL_MLA        => self.execute_mul_mla(inst),
            ArmOpcode::MULL_MLAL      => self.execute_mull_mlal(inst),
            ArmOpcode::DataProcessing => self.execute_data_processing(inst),
            ArmOpcode::MRS            => self.execute_mrs(inst),
            ArmOpcode::MSR_Reg        => self.execute_msr_reg(inst),
            _ => unimplemented!(),
        };

        Ok(())
    }

    fn execute_bx(&mut self, inst: ArmInstruction) {
        self.clear_pipeline();
        let addr = self.gpr[inst.Rm()] as u32;
        self.state = if (addr & 0b1) == 0 { State::ARM } else { State::THUMB };
        self.cpsr.set_state(self.state);
        self.gpr[15] = (addr & 0xFFFFFFFE) as i32;
        // TODO missaligned PC in ARM state?
    }

    fn execute_b_bl(&mut self, inst: ArmInstruction) {
        self.clear_pipeline();
        if inst.is_branch_with_link() { self.gpr[14] = self.gpr[15].wrapping_sub(4); }
        self.gpr[15] = self.gpr[15].wrapping_add(inst.branch_offset());
    }

    fn execute_mul_mla(&mut self, inst: ArmInstruction) {
        if inst.is_setting_flags() { return self.execute_mul_mla_s(inst); }
        let mut res = self.gpr[inst.Rs()].wrapping_mul(self.gpr[inst.Rm()]);
        if inst.is_accumulating() { res = res.wrapping_add(self.gpr[inst.Rd()]); }
        self.gpr[inst.Rn()] = res;
    }

    fn execute_mul_mla_s(&mut self, inst: ArmInstruction) {
        let mut res = (self.gpr[inst.Rs()] as u64).wrapping_mul(self.gpr[inst.Rm()] as u64);
        if inst.is_accumulating() { res = res.wrapping_add(self.gpr[inst.Rd()] as u64); }
        let x = (res & 0x00000000_FFFFFFFF_u64) as i32;
        self.gpr[inst.Rn()] = x;
        self.update_flags(x, res);
        self.cpsr.set_V(false); // Does not set V.
    }

    fn execute_mull_mlal(&mut self, inst: ArmInstruction) {
        let mut res: u64 = if inst.is_signed() {
            (self.gpr[inst.Rs()] as i64).wrapping_mul(self.gpr[inst.Rm()] as i64) as u64
        } else {
            (self.gpr[inst.Rs()] as u64).wrapping_mul(self.gpr[inst.Rm()] as u64)
        };
        if inst.is_accumulating() {
            res = res.wrapping_add(((self.gpr[inst.Rn()] as u64) << 32) | (self.gpr[inst.Rd()] as u64));
        }
        self.gpr[inst.Rn()] = ((res >> 32) & (u32::MAX as u64)) as i32;
        self.gpr[inst.Rd()] = ((res >>  0) & (u32::MAX as u64)) as i32;

        if inst.is_setting_flags() {
            self.cpsr.set_N((res as i64) < 0);
            self.cpsr.set_Z(res == 0);
            self.cpsr.set_C((res & (1 << 32)) != 0);  // Unpredictable, i.e. do what you want.
            self.cpsr.set_V(res > (u32::MAX as u64)); // Unpredictable, i.e. do what you want.
        }
    }

    fn execute_data_processing(&mut self, inst: ArmInstruction) {
        if inst.is_setting_flags() { return self.execute_data_processing_s(inst); }
        let op2: i32 = inst.calculate_shft_field(&self.gpr[..], self.cpsr.C());
        let rn: i32 = self.gpr[inst.Rn()];
        let rd: &mut i32 = &mut self.gpr[inst.Rd()];
        let c: i32 = if self.cpsr.C() { 1 } else { 0 };

        match inst.dpop() {
            ArmDPOP::AND => { *rd = rn & op2; },
            ArmDPOP::EOR => { *rd = rn ^ op2; },
            ArmDPOP::SUB => { *rd = rn.wrapping_sub(op2); },
            ArmDPOP::RSB => { *rd = op2.wrapping_sub(rn); },
            ArmDPOP::ADD => { *rd = rn.wrapping_add(op2); },
            ArmDPOP::ADC => { *rd = rn.wrapping_add(op2).wrapping_add(c) },
            ArmDPOP::SBC => { *rd = rn.wrapping_sub(op2).wrapping_sub(1-c); },
            ArmDPOP::RSC => { *rd = op2.wrapping_sub(rn).wrapping_sub(1-c); },
            ArmDPOP::TST => panic!("S bit for TST instruction not set!"),
            ArmDPOP::TEQ => panic!("S bit for TEQ instruction not set!"),
            ArmDPOP::CMP => panic!("S bit for CMP instruction not set!"),
            ArmDPOP::CMN => panic!("S bit for CMN instruction not set!"),
            ArmDPOP::ORR => { *rd = rn | op2; },
            ArmDPOP::MOV => { *rd = op2; },
            ArmDPOP::BIC => { *rd = rn & !op2; },
            ArmDPOP::MVN => { *rd = !op2; },
        }
    }

    fn execute_data_processing_s(&mut self, inst: ArmInstruction) {
        // TODO this code needs improvement!
        let (op2, cshft) = inst.calculate_shft_field_with_carry(&self.gpr[..], self.cpsr.C());
        let op2 = op2 as u64;
        let rn: u64 = self.gpr[inst.Rn()] as u64;
        let c: u64 = if self.cpsr.C() { 1 } else { 0 }; // TODO cshft or CPSR.C?
        let mut late_cv = true;
        let mut no_wb   = false;
        let mut lspsr   = false;

        let res: u64 = match inst.dpop() {
            ArmDPOP::AND => { late_cv = false; self.cpsr.set_C(cshft); rn & op2 },
            ArmDPOP::EOR => { late_cv = false; self.cpsr.set_C(cshft); rn ^ op2 },
            ArmDPOP::SUB => { rn.wrapping_sub(op2) },
            ArmDPOP::RSB => { op2.wrapping_sub(rn) },
            ArmDPOP::ADD => { rn.wrapping_add(op2) },
            ArmDPOP::ADC => { rn.wrapping_add(op2).wrapping_add(c) },
            ArmDPOP::SBC => { rn.wrapping_sub(op2).wrapping_sub(1-c) },
            ArmDPOP::RSC => { op2.wrapping_sub(rn).wrapping_sub(1-c) },
            ArmDPOP::TST => { no_wb = true; late_cv = false; self.cpsr.set_C(cshft); rn & op2 },
            ArmDPOP::TEQ => { no_wb = true; late_cv = false; self.cpsr.set_C(cshft); rn ^ op2 },
            ArmDPOP::CMP => { no_wb = true; rn.wrapping_sub(op2) },
            ArmDPOP::CMN => { no_wb = true; rn.wrapping_add(op2) },
            ArmDPOP::ORR => { late_cv = false; self.cpsr.set_C(cshft); rn | op2 },
            ArmDPOP::MOV => { lspsr = inst.Rd() == Arm7Tdmi::PC; op2 },
            ArmDPOP::BIC => { late_cv = false; self.cpsr.set_C(cshft); rn & !op2 },
            ArmDPOP::MVN => { lspsr = inst.Rd() == Arm7Tdmi::PC; !op2 },
        };

        let rd = (res & (u32::MAX as u64)) as i32;

        if lspsr { self.cpsr = CPSR(self.spsr[self.mode as u8 as usize]); }
        else {
            self.cpsr.set_N(rd < 0);
            self.cpsr.set_Z(rd == 0);
            if late_cv {
                self.cpsr.set_C(0 != (res & (1 << 32)));
                self.cpsr.set_V(res > (u32::MAX as u64));
            }
        }

        if !no_wb { self.gpr[inst.Rd()] = rd; }
    }

    fn execute_mrs(&mut self, inst: ArmInstruction) {
        if inst.is_accessing_spsr() {
            self.gpr[inst.Rd()] = self.spsr[self.mode as u8 as usize] as i32;
        } else {
            self.gpr[inst.Rd()] = self.cpsr.0 as i32;
        }
    }

    fn execute_msr_reg(&mut self, inst: ArmInstruction) {
        // TODO decoding seems to be wrong? (fields)
        unimplemented!()
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
