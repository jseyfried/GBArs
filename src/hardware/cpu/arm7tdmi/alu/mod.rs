// License below.
//! Implements common execution logic for THUMB and ARM state instruction.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use super::*;

pub use self::armdpop::*;

pub mod armdpop;

impl Arm7Tdmi {
    fn alu_data_processing(&mut self, dpop: ArmDPOP, op1: i32, op2: i32) -> i32 {
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
        let mut vf = self.cpsr.V();

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
