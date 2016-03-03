// License below.
//! Implements the 4-bit condition field of an ARM/THUMB instruction.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use super::super::error::GbaError;
use super::arm7tdmi::CPSR;

/// The condition field of an ARM instruction.
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum ArmCondition {
    #[doc = "Z set. EQual."]                                       EQ = 0b0000,
    #[doc = "Z clear. Not Equal."]                                 NE = 0b0001,
    #[doc = "C set. Unsigned Higher or Same."]                     HS = 0b0010,
    #[doc = "C clear. Unsigned LOwer."]                            LO = 0b0011,
    #[doc = "N set. MInus, i.e. negative."]                        MI = 0b0100,
    #[doc = "N clear. PLus, i.e. positive or zero."]               PL = 0b0101,
    #[doc = "V Set. Overflow."]                                    VS = 0b0110,
    #[doc = "V Clear. No Overflow."]                               VC = 0b0111,
    #[doc = "C set and Z clear. Unsigned HIgher."]                 HI = 0b1000,
    #[doc = "C clear or Z set. Unsigned Lower or Same."]           LS = 0b1001,
    #[doc = "N equals V. Greater than or Equal to."]               GE = 0b1010,
    #[doc = "N distinct from V. Less Than."]                       LT = 0b1011,
    #[doc = "Z clear and N equals V. Greater Than."]               GT = 0b1100,
    #[doc = "Z set or N distinct from V.  Less than or Equal to."] LE = 0b1101,
    #[doc = "ALways execute this instruction, i.e. no condition."] AL = 0b1110,
    #[doc = "Reserved."]                                           NV = 0b1111,
}

impl ArmCondition {
    /// Evaluates the condition field depending on the CPSR's state.
    ///
    /// # Params
    /// - `cpsr`: The CPSR to inspect.
    ///
    /// # Returns
    /// - `Ok`: `true` if the corresponding instruction should be executed, otherwise `false`.
    /// - `Err`: The condition field is `NV`, which is reserved in ARM7TDMI.
    pub fn check(self, cpsr: &CPSR) -> Result<bool, GbaError> {
        match self {
            ArmCondition::EQ => Ok( cpsr.Z() ),
            ArmCondition::NE => Ok(!cpsr.Z() ),
            ArmCondition::HS => Ok( cpsr.C() ),
            ArmCondition::LO => Ok(!cpsr.C() ),
            ArmCondition::MI => Ok( cpsr.N() ),
            ArmCondition::PL => Ok(!cpsr.N() ),
            ArmCondition::VS => Ok( cpsr.V() ),
            ArmCondition::VC => Ok(!cpsr.V() ),
            ArmCondition::HI => Ok( cpsr.C() & !cpsr.Z() ),
            ArmCondition::LS => Ok(!cpsr.C() |  cpsr.Z() ),
            ArmCondition::GE => Ok( cpsr.N() == cpsr.V() ),
            ArmCondition::LT => Ok( cpsr.N() != cpsr.V() ),
            ArmCondition::GT => Ok(!cpsr.Z() & (cpsr.N() == cpsr.V()) ),
            ArmCondition::LE => Ok( cpsr.Z() | (cpsr.N() != cpsr.V()) ),
            ArmCondition::AL => Ok( true ),
            ArmCondition::NV => Err(GbaError::ReservedArmConditionNV),
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
