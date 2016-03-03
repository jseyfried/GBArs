// License below.
//! Implements data processing opcodes for ARM state instructions.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use std::fmt;

/// A data processing opcode.
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum ArmDPOP {
    #[doc = "Bitwise AND"]                  AND = 0b0000,
    #[doc = "Bitwise XOR"]                  EOR = 0b0001,
    #[doc = "Subtraction"]                  SUB = 0b0010,
    #[doc = "Reverse subtraction"]          RSB = 0b0011,
    #[doc = "Addition"]                     ADD = 0b0100,
    #[doc = "Add with carry"]               ADC = 0b0101,
    #[doc = "Subtract with borrow"]         SBC = 0b0110,
    #[doc = "Reverse subtract with borrow"] RSC = 0b0111,
    #[doc = "Test bits"]                    TST = 0b1000,
    #[doc = "Test bitwise equality"]        TEQ = 0b1001,
    #[doc = "Compare"]                      CMP = 0b1010,
    #[doc = "Compare negative"]             CMN = 0b1011,
    #[doc = "Bitwise OR"]                   ORR = 0b1100,
    #[doc = "Move value"]                   MOV = 0b1101,
    #[doc = "Bit clear"]                    BIC = 0b1110,
    #[doc = "Move bitwise negated value"]   MVN = 0b1111,
}

impl ArmDPOP {
    /// Checks whether this instruction does not
    /// write any results to a destination register.
    pub fn is_test(self) -> bool {
        match self {
            ArmDPOP::TST | ArmDPOP::TEQ | ArmDPOP::CMP | ArmDPOP::CMN => true,
            _ => false,
        }
    }

    /// Checks whether this instruction is a
    /// move instruction.
    pub fn is_move(self) -> bool {
        match self {
            ArmDPOP::MOV | ArmDPOP::MVN => true,
            _ => false,
        }
    }
}

impl fmt::Display for ArmDPOP {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ArmDPOP::AND => write!(f, "and"),
            ArmDPOP::EOR => write!(f, "eor"),
            ArmDPOP::SUB => write!(f, "sub"),
            ArmDPOP::RSB => write!(f, "rsb"),
            ArmDPOP::ADD => write!(f, "add"),
            ArmDPOP::ADC => write!(f, "adc"),
            ArmDPOP::SBC => write!(f, "sbc"),
            ArmDPOP::RSC => write!(f, "rsc"),
            ArmDPOP::TST => write!(f, "tst"),
            ArmDPOP::TEQ => write!(f, "teq"),
            ArmDPOP::CMP => write!(f, "cmp"),
            ArmDPOP::CMN => write!(f, "cmn"),
            ArmDPOP::ORR => write!(f, "orr"),
            ArmDPOP::MOV => write!(f, "mov"),
            ArmDPOP::BIC => write!(f, "bic"),
            ArmDPOP::MVN => write!(f, "mvn"),
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
