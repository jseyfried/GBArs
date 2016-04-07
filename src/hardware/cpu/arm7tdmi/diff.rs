// License below.
//! Implements a diff viewer for the ARM7TDMI's register state.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use super::Arm7Tdmi;
use super::psr::{PSR, State, Mode};
use super::super::arminstruction::ArmInstruction;
use super::super::thumbinstruction::ThumbInstruction;
use std::io;
use super::super::super::super::term;

/// A diff viewer for the ARM7TDMI.
///
/// This thingy takes the current state of a CPU
/// emulator and prints them to the terminal, while
/// optionally highlighting changed values with colours.
pub struct Arm7TdmiDiff {
    // Register state.
    cpsr_old: PSR,
    cpsr_new: PSR,
    spsr_old: PSR,
    spsr_new: PSR,
    gpr: [i32; 16],
    gpr_new: u16,

    // Pipeline state.
    fetched_arm: u32,
    decoded_arm: ArmInstruction,
    fetched_thumb: u16,
    decoded_thumb: ThumbInstruction,

    // Diff config.
    colour: bool,
}

impl Arm7TdmiDiff {
    /// The colour in which changed values will be shown.
    pub const DIFF_COLOUR: term::color::Color = term::color::BRIGHT_YELLOW;

    /// The colour in which head lines will appear.
    pub const HEAD_COLOUR: term::color::Color = term::color::BRIGHT_BLUE;

    const DEBUG_REGISTER_NAMES: &'static [&'static str] = &[
        "R0:  ", "R1:  ", "R2:  ", "R3:  ", "R4:  ", "R5:  ", "R6:  ", "R7:  ",
        "R8:  ", "R9:  ", "R10: ", "R11: ", "R12: ", "SP:  ", "LR:  ", "PC:  "
    ];

    /// Creates a default initialised diff.
    pub fn new() -> Arm7TdmiDiff {
        Arm7TdmiDiff {
            cpsr_old: PSR::default(),
            cpsr_new: PSR::default(),
            spsr_old: PSR::default(),
            spsr_new: PSR::default(),
            gpr: [0; 16],
            gpr_new: 0,
            fetched_arm: ArmInstruction::NOP_RAW,
            decoded_arm: ArmInstruction::nop(),
            fetched_thumb: ThumbInstruction::NOP_RAW,
            decoded_thumb: ThumbInstruction::nop(),
            colour: true,
        }
    }

    /// Configure whether `print` should colourise differences.
    pub fn set_colourising(&mut self, c: bool) { self.colour = c; }

    /// Check whether `print` currently colourises differences.
    pub fn is_colourising(&self) -> bool { self.colour }

    /// Applys the new CPU state to this diff marking changes.
    pub fn diff(&mut self, cpu: &Arm7Tdmi) {
        self.cpsr_old = self.cpsr_new;
        self.spsr_old = self.spsr_new;
        self.cpsr_new = cpu.cpsr;
        self.spsr_new = cpu.spsr[cpu.cpsr.mode() as u8 as usize];
        self.gpr_new = 0;
        for i in 0_u32..16 {
            let j = i as usize;
            if self.gpr[j] != cpu.gpr[j] {
                self.gpr_new |= 1 << i;
                self.gpr[j] = cpu.gpr[j];
            }
        }
        self.fetched_arm   = cpu.fetched_arm;
        self.decoded_arm   = cpu.decoded_arm;
        self.fetched_thumb = cpu.fetched_thumb;
        self.decoded_thumb = cpu.decoded_thumb;
    }

    /// Prints the current registers of an Arm7Tdmi where
    /// changed values are colourised.
    pub fn print(&self, terminal: &mut Box<term::StdoutTerminal>) -> io::Result<()> {
        terminal.reset().unwrap_or(());
        Arm7TdmiDiff::colourise_head(terminal, self.colour);
        try!(write!(terminal, "# Arm7Tdmi\n\t- Register Set"));
        terminal.reset().unwrap_or(());

        // Write PSRs.
        try!(write!(terminal, "\n\t\tCPSR: "));
        try!(Arm7TdmiDiff::print_psr(terminal, self.cpsr_old, self.cpsr_new, self.colour));
        try!(write!(terminal, "\tSPSR: "));
        if self.cpsr_new.mode() == Mode::User { try!(write!(terminal, "[---- -- ----- ---]")); }
        else { try!(Arm7TdmiDiff::print_psr(terminal, self.spsr_old, self.spsr_new, self.colour)); }
        try!(write!(terminal, "\n"));

        // Write GPRs.
        for i in 0..16 {
            if (i % 4) == 0 { try!(write!(terminal, "\n\t\t")); }
            try!(write!(terminal, "{}[", Arm7TdmiDiff::DEBUG_REGISTER_NAMES[i]));
            if 0 != (self.gpr_new & (1 << i)) { Arm7TdmiDiff::colourise_diff(terminal, self.colour); }
            try!(write!(terminal, "{:08X}", self.gpr[i]));
            terminal.reset().unwrap_or(());
            try!(write!(terminal, "]\t"));
        }

        // Write pipeline state.
        Arm7TdmiDiff::colourise_head(terminal, self.colour);
        try!(write!(terminal, "\n\n\t- Pipeline State"));
        terminal.reset().unwrap_or(());
        if self.cpsr_new.state() == State::ARM {
            try!(write!(terminal, "\n\t\tARM   Fetch:  {:#010X}\n\t\t\
                                         ARM   Decode: {}\n", self.fetched_arm, self.decoded_arm));
        } else {
            try!(write!(terminal, "\n\t\tTHUMB Fetch:  {:#06X}\n\t\t\
                                         THUMB Decode: {}\n", self.fetched_thumb, self.decoded_thumb));
        }

        terminal.reset().unwrap_or(());
        write!(terminal, "\n")
    }

    fn print_psr(terminal: &mut Box<term::StdoutTerminal>, old: PSR, new: PSR, colour: bool) -> io::Result<()> {
        // TODO macro to simplify this code
        try!(write!(terminal, "["));

        // Write flags.
        if new.N() != old.N() { Arm7TdmiDiff::colourise_diff(terminal, colour); }
        else { terminal.reset().unwrap_or(()); }
        try!(write!(terminal, "{}", if new.N() { 'N' } else { 'n' }));

        if new.Z() != old.Z() { Arm7TdmiDiff::colourise_diff(terminal, colour); }
        else { terminal.reset().unwrap_or(()); }
        try!(write!(terminal, "{}", if new.Z() { 'Z' } else { 'z' }));

        if new.C() != old.C() { Arm7TdmiDiff::colourise_diff(terminal, colour); }
        else { terminal.reset().unwrap_or(()); }
        try!(write!(terminal, "{}", if new.C() { 'C' } else { 'c' }));

        if new.V() != old.V() { Arm7TdmiDiff::colourise_diff(terminal, colour); }
        else { terminal.reset().unwrap_or(()); }
        try!(write!(terminal, "{}", if new.V() { 'V' } else { 'v' }));

        // Write interrupt flags.
        try!(write!(terminal, " "));

        if new.irq_disabled() != old.irq_disabled() { Arm7TdmiDiff::colourise_diff(terminal, colour); }
        else { terminal.reset().unwrap_or(()); }
        try!(write!(terminal, "{}", if new.irq_disabled() { 'I' } else { 'i' }));

        if new.fiq_disabled() != old.fiq_disabled() { Arm7TdmiDiff::colourise_diff(terminal, colour); }
        else { terminal.reset().unwrap_or(()); }
        try!(write!(terminal, "{}", if new.fiq_disabled() { 'F' } else { 'f' }));

        // Write state and mode.
        try!(write!(terminal, " "));

        if new.state() != old.state() { Arm7TdmiDiff::colourise_diff(terminal, colour); }
        else { terminal.reset().unwrap_or(()); }
        try!(write!(terminal, "{} ", new.state()));

        if new.mode() != old.mode() { Arm7TdmiDiff::colourise_diff(terminal, colour); }
        else { terminal.reset().unwrap_or(()); }
        try!(write!(terminal, "{}", new.mode()));

        // Done.
        terminal.reset().unwrap_or(());
        write!(terminal, "]")
    }

    fn colourise_diff(terminal: &mut Box<term::StdoutTerminal>, colour: bool) {
        if colour { terminal.fg(Arm7TdmiDiff::DIFF_COLOUR).unwrap_or(()); }
    }

    fn colourise_head(terminal: &mut Box<term::StdoutTerminal>, colour: bool) {
        if colour { terminal.fg(Arm7TdmiDiff::HEAD_COLOUR).unwrap_or(()); }
    }
}

impl Default for Arm7TdmiDiff { fn default() -> Arm7TdmiDiff { Arm7TdmiDiff::new() } }


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
