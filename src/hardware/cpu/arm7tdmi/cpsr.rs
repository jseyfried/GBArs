// License below.
//! Implements the *"Current Program Status Register"* (CPSR) of the ARM7TDMI.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use std::fmt;
use std::mem;

/// The CPU's instruction decoding states.
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum State {
    /// Currently executing 32-bit ARM instructions.
    ARM = 0,

    /// Currently executing 16-bit THUMB instructions.
    THUMB,
}


/// The CPU's different execution modes.
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum Mode {
    #[doc = "CPU mode for running normal user code."]                  User = 0,
    #[doc = "CPU mode for handling fast interrupts."]                  FIQ,
    #[doc = "CPU mode for handling normal interrupts."]                IRQ,
    #[doc = "CPU mode for executing supervisor code."]                 Supervisor,
    #[doc = "CPU mode entered if memory lookups are aborted."]         Abort,
    #[doc = "CPU mode entered if executing an undefined instruction."] Undefined,
    #[doc = "CPU mode for executing system code."]                     System,
}

impl Mode {
    /// Converts this mode into a CPSR bit pattern.
    pub fn as_bits(self) -> u32 {
        match self {
            Mode::User       => CPSR::MODE_USER,
            Mode::FIQ        => CPSR::MODE_FIQ,
            Mode::IRQ        => CPSR::MODE_IRQ,
            Mode::Supervisor => CPSR::MODE_SUPERVISOR,
            Mode::Abort      => CPSR::MODE_ABORT,
            Mode::Undefined  => CPSR::MODE_UNDEFINED,
            Mode::System     => CPSR::MODE_SYSTEM
        }
    }
}


/// The Current Program Status Register.
#[derive(PartialEq, Clone, Copy)]
pub struct CPSR(pub u32);

impl CPSR {
    /// Used to mask reserved bits away.
    pub const NON_RESERVED_MASK: u32 = 0b11110000_00000000_00000000_11111111_u32;
    //                                   NZCV                       IFTMMMMM

    /// Masks the flag bits of a PSR.
    pub const FLAGS_MASK: u32 = 0xF0000000_u32;

    /// Sign flag bit.
    ///
    /// 1 if signed, otherwise 0.
    pub const SIGN_FLAG_BIT: u8 = 31;

    /// Zero flag bit.
    ///
    /// 1 if zero, otherwise 0.
    pub const ZERO_FLAG_BIT: u8 = 30;

    /// Carry flag bit.
    ///
    /// 1 if carry or no borrow, 0 if borrow or no carry.
    pub const CARRY_FLAG_BIT: u8 = 29;

    /// Overflow flag bit.
    ///
    /// 1 if overflow, otherwise 0.
    pub const OVERFLOW_FLAG_BIT: u8 = 28;

    /// IRQ disable bit.
    ///
    /// 1 if disabled, otherwise 0.
    pub const IRQ_DISABLE_BIT: u8 = 7;

    /// FIQ disable bit.
    ///
    /// 1 if disabled, otherwise 0.
    pub const FIQ_DISABLE_BIT: u8 = 6;

    /// State bit.
    ///
    /// 1 if THUMB, 0 if ARM.
    pub const STATE_BIT: u8 = 5;

    /// Mode bits mask.
    ///
    /// Used to get the mode bits only.
    pub const MODE_MASK: u32 = 0b0001_1111;

    /// Bit pattern for user mode.
    pub const MODE_USER: u32 = 0b1_0000;

    /// Bit pattern for FIQ mode.
    pub const MODE_FIQ: u32 = 0b1_0001;

    /// Bit pattern for IRQ mode.
    pub const MODE_IRQ: u32 = 0b1_0010;

    /// Bit pattern for supervisor mode.
    pub const MODE_SUPERVISOR: u32 = 0b1_0011;

    /// Bit pattern for abort mode.
    pub const MODE_ABORT: u32 = 0b1_0111;

    /// Bit pattern for undefined mode.
    pub const MODE_UNDEFINED: u32 = 0b1_1011;

    /// Bit pattern for system mode.
    pub const MODE_SYSTEM: u32 = 0b1_1111;


    /// Clears all reserved bits.
    pub fn clear_reserved_bits(&mut self) {
        self.0 &= CPSR::NON_RESERVED_MASK;
    }

    /// Get the condition bits.
    ///
    /// # Returns
    /// The condition bits are laid out as such:
    /// ```
    /// 0b0000
    /// //NZCV
    /// ```
    pub fn condition_bits(&self) -> u32 {
        (self.0 as u32) >> CPSR::OVERFLOW_FLAG_BIT
    }

    /// Converts the state bit to a state enum.
    pub fn state(&self) -> State {
        unsafe { mem::transmute(((self.0 >> CPSR::STATE_BIT) & 1) as u8) }
    }

    /// Converts the mode bit pattern to a mode enum.
    pub fn mode(&self) -> Mode {
        match self.0 & CPSR::MODE_MASK {
            CPSR::MODE_USER       => Mode::User,
            CPSR::MODE_FIQ        => Mode::FIQ,
            CPSR::MODE_IRQ        => Mode::IRQ,
            CPSR::MODE_SUPERVISOR => Mode::Supervisor,
            CPSR::MODE_ABORT      => Mode::Abort,
            CPSR::MODE_UNDEFINED  => Mode::Undefined,
            CPSR::MODE_SYSTEM     => Mode::System,
            _ => {
                error!("CPSR: Unrecognised mode bit pattern {:#8b}.", self.0 & CPSR::MODE_MASK);
                panic!("Aborting due to illegal mode bits.");
            },
        }
    }

    /// Sets or clears the state bit
    /// depending on the new state.
    pub fn set_state(&mut self, s: State) {
        self.0 &= !(1 << CPSR::STATE_BIT);
        self.0 |= (s as u8 as u32) << CPSR::STATE_BIT;
    }

    /// Sets or clears the mode bits
    /// depending on the new mode.
    pub fn set_mode(&mut self, m: Mode) {
        self.0 &= !CPSR::MODE_MASK;
        self.0 |= m.as_bits();
    }

    /// Sets the IRQ disable bit.
    pub fn disable_irq(&mut self) {
        self.0 |= 1 << CPSR::IRQ_DISABLE_BIT;
    }

    /// Sets the FIQ disable bit.
    pub fn disable_fiq(&mut self) {
        self.0 |= 1 << CPSR::FIQ_DISABLE_BIT;
    }

    /// Clears the IRQ disable bit.
    pub fn enable_irq(&mut self) {
        self.0 &= !(1 << CPSR::IRQ_DISABLE_BIT);
    }

    /// Clears the FIQ disable bit.
    pub fn enable_fiq(&mut self) {
        self.0 &= !(1 << CPSR::FIQ_DISABLE_BIT);
    }

    /// Gets the current state of the IRQ disable bit.
    pub fn irq_disabled(&self) -> bool {
        0 != (self.0 & (1 << CPSR::IRQ_DISABLE_BIT))
    }

    /// Gets the current state of the FIQ disable bit.
    pub fn fiq_disabled(&self) -> bool {
        0 != (self.0 & (1 << CPSR::FIQ_DISABLE_BIT))
    }

    /// Gets the current state of the N bit.
    #[allow(non_snake_case)]
    pub fn N(self) -> bool { 0 != (self.0 & (1 << 31)) }

    /// Gets the current state of the Z bit.
    #[allow(non_snake_case)]
    pub fn Z(self) -> bool { 0 != (self.0 & (1 << 30)) }

    /// Gets the current state of the C bit.
    #[allow(non_snake_case)]
    pub fn C(self) -> bool { 0 != (self.0 & (1 << 29)) }

    /// Gets the current state of the V bit.
    #[allow(non_snake_case)]
    pub fn V(self) -> bool { 0 != (self.0 & (1 << 28)) }

    /// Set the new state of the N bit.
    #[allow(non_snake_case)]
    pub fn set_N(&mut self, n: bool) { if n { self.0 |= 1 << 31; } else { self.0 &= !(1 << 31); } }

    /// Set the new state of the Z bit.
    #[allow(non_snake_case)]
    pub fn set_Z(&mut self, n: bool) { if n { self.0 |= 1 << 30; } else { self.0 &= !(1 << 30); } }

    /// Set the new state of the C bit.
    #[allow(non_snake_case)]
    pub fn set_C(&mut self, n: bool) { if n { self.0 |= 1 << 29; } else { self.0 &= !(1 << 29); } }

    /// Set the new state of the V bit.
    #[allow(non_snake_case)]
    pub fn set_V(&mut self, n: bool) { if n { self.0 |= 1 << 28; } else { self.0 &= !(1 << 28); } }
}

impl fmt::Display for CPSR {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{n}{z}{c}{v} {i}{f} {s} {m}]",
            n = if self.N() { 'N' } else { 'n' },
            z = if self.Z() { 'Z' } else { 'z' },
            c = if self.C() { 'C' } else { 'c' },
            v = if self.V() { 'V' } else { 'v' },
            i = if self.irq_disabled() { 'I' } else { 'i' },
            f = if self.fiq_disabled() { 'F' } else { 'f' },
            s = self.state(), m = self.mode()
        )
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", if *self == State::ARM { "ARM" } else { "THUMB" })
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
            match *self {
                Mode::User       => "USR",
                Mode::FIQ        => "FIQ",
                Mode::IRQ        => "IRQ",
                Mode::Supervisor => "SVC",
                Mode::Abort      => "ABT",
                Mode::Undefined  => "UND",
                Mode::System     => "SYS",
            }
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
