// License below.
//! Provides an error type for GBA hardware emulation errors.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use std::error;
use std::fmt;


/// An error caused during GBA hardware emulation.
#[derive(Debug)]
pub enum GbaError {
    /// An invalid ARM instruction has been decoded.
    ///
    /// This is not to be confused with an undefined instruction.
    InvalidArmInstruction(u32),

    /// An invalid THUMB instruction has been decoded.
    InvalidThumbInstruction(u16),

    /// An instruction using the reserved `NV` condition has been executed.
    ReservedArmConditionNV,

    /// Tried accessing an invalid physical address.
    InvalidPhysicalAddress(u32),
}

impl error::Error for GbaError {
    fn description(&self) -> &str {
        match *self {
            GbaError::InvalidArmInstruction(_)   => "Invalid instruction in ARM state.",
            GbaError::InvalidThumbInstruction(_) => "Invalid instruction in THUMB state.",
            GbaError::ReservedArmConditionNV     => "Invalid NV condition in ARM state.",
            GbaError::InvalidPhysicalAddress(_)  => "Invalid physical address."
        }
    }
}

impl fmt::Display for GbaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GbaError::InvalidArmInstruction(x)   => write!(f, "Invalid ARM Instruction {:#010X}", x),
            GbaError::InvalidThumbInstruction(x) => write!(f, "Invalid THUMB Instruction {:#06X}", x),
            GbaError::ReservedArmConditionNV     => write!(f, "Invalid ARM condition NV"),
            GbaError::InvalidPhysicalAddress(x)  => write!(f, "Invalid physical address {:#010X}", x),
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
