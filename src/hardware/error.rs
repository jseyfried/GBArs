// License below.
//! Provides an error type for GBA hardware emulation errors.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use std::error;
use std::fmt;


/// An error caused during GBA hardware emulation.
#[derive(Debug, PartialEq, Clone)]
pub enum GbaError {
    /// An invalid ARM instruction has been decoded.
    ///
    /// This is not to be confused with an undefined instruction
    /// or an instruction doing illegal stuff.
    InvalidArmInstruction(u32),

    /// An invalid THUMB instruction has been decoded.
    ///
    /// This is not to be confused with an undefined instruction
    /// or an instruction doing illegal stuff.
    InvalidThumbInstruction(u16),

    /// An instruction using the reserved `NV` condition has been executed.
    ReservedArmConditionNV,

    /// Tried accessing an invalid physical address.
    InvalidPhysicalAddress(u32),

    /// Tried writing to a ROM.
    InvalidRomAccess(u32),

    /// Tried accessing `.1` bits data at the memory location `.0`.
    InvalidMemoryBusWidth(u32, u8),

    /// An instruction illegally writes to or reads from PC.
    ///
    /// Note that this does not include instructions where it is
    /// just advised to not do that due to unpredictable behaviour.
    InvalidUseOfR15,

    /// Registers that should be distinct are the same.
    InvalidRegisterReuse(usize, usize, usize, usize),

    /// Writing an offset back to a base register where the instruction shouldn't.
    InvalidOffsetWriteBack,

    /// Tried executing a privileged instruction in user mode.
    PrivilegedUserCode,
}

impl error::Error for GbaError {
    fn description(&self) -> &str {
        match *self {
            GbaError::InvalidArmInstruction(_)      => "Invalid instruction in ARM state.",
            GbaError::InvalidThumbInstruction(_)    => "Invalid instruction in THUMB state.",
            GbaError::ReservedArmConditionNV        => "Invalid NV condition in ARM state.",
            GbaError::InvalidPhysicalAddress(_)     => "Invalid physical address.",
            GbaError::InvalidRomAccess(_)           => "Invalid write attempt to a ROM.",
            GbaError::InvalidMemoryBusWidth(_,_)    => "Invalid bus width while accessing memory.",
            GbaError::InvalidUseOfR15               => "Invalid use of PC in an instruction.",
            GbaError::InvalidRegisterReuse(_,_,_,_) => "Invalid re-use of registers in an instruction.",
            GbaError::InvalidOffsetWriteBack        => "Invalid write-back of an offset to a base register.",
            GbaError::PrivilegedUserCode            => "Invalid privileged instruction in user mode.",
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
            GbaError::InvalidRomAccess(x)        => write!(f, "Invalid write attempt to a ROM at {:#010X}", x),
            GbaError::InvalidMemoryBusWidth(x,w) => write!(f, "Invalid {}-bit bus while accessing memory at {:#010X}", w, x),
            GbaError::InvalidUseOfR15            => write!(f, "Invalid use of PC in an instruction."),
            GbaError::InvalidOffsetWriteBack     => write!(f, "Invalid write-back of an offset to a base register."),
            GbaError::PrivilegedUserCode         => write!(f, "Invalid privileged instruction in user mode."),
            GbaError::InvalidRegisterReuse(n,d,s,m) => {
                write!(f, "Invalid re-use of the same register. Rn={}, Rd={}, Rs={}, Rm={}", n, d, s, m)
            },
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
