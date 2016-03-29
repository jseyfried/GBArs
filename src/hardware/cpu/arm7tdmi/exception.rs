// License below.
//! Implements exceptions caused in the ARM7TDMI.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use super::psr::Mode;

/// CPU exceptions.
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum Exception {
    #[doc = "Exception due to resetting the CPU."]                Reset = 0,
    #[doc = "Exception due to executing undefined instructions."] UndefinedInstruction,
    #[doc = "Exception due to executing SWI."]                    SoftwareInterrupt,
    #[doc = "Instruction prefetching aborted."]                   PrefetchAbort,
    #[doc = "Data prefetching aborted."]                          DataAbort,
    #[doc = "Exception due to resolving large addresses."]        AddressExceeds26Bit,
    #[doc = "Exception due to a normal hardware interrupt."]      NormalInterrupt,
    #[doc = "Exception due to a fast hardware interrupt."]        FastInterrupt,
}

impl Exception {
    /// Get the exception's priority.
    ///
    /// # Returns
    /// 1 = highest, 7 = lowest.
    pub fn priority(self) -> u8 {
        match self {
            Exception::AddressExceeds26Bit |
            Exception::FastInterrupt        => 3,
            Exception::Reset                => 1,
            Exception::UndefinedInstruction => 7,
            Exception::SoftwareInterrupt    => 6,
            Exception::PrefetchAbort        => 5,
            Exception::DataAbort            => 2,
            Exception::NormalInterrupt      => 4,
        }
    }

    /// Get the exception's CPU mode on entry.
    pub fn mode_on_entry(self) -> Mode {
        match self {
            Exception::PrefetchAbort |
            Exception::DataAbort            => Mode::Abort,
            Exception::Reset |
            Exception::SoftwareInterrupt |
            Exception::AddressExceeds26Bit  => Mode::Supervisor,
            Exception::UndefinedInstruction => Mode::Undefined,
            Exception::NormalInterrupt      => Mode::IRQ,
            Exception::FastInterrupt        => Mode::FIQ,
        }
    }

    /// Check whether fast interrupts should be disabled.
    ///
    /// # Returns
    /// - `true` if FIQ should be disabled on entry.
    /// - `false` if FIQ should be left unchanged.
    #[cfg_attr(feature="clippy", allow(inline_always))]
    #[inline(always)]
    pub fn disable_fiq_on_entry(self) -> bool {
        (self == Exception::Reset) | (self == Exception::FastInterrupt)
    }

    /// Get the exception vector address.
    ///
    /// # Returns
    /// A physical address to the exception's
    /// vector entry.
    #[cfg_attr(feature="clippy", allow(inline_always))]
    #[inline(always)]
    pub fn vector_address(self) -> u32 {
        (self as u8 as u32) * 4
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
