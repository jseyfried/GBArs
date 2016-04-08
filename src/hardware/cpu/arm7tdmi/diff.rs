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
use super::super::super::super::term_painter::ToStyle;
use super::super::super::super::term_painter::Color::*;
use super::super::super::super::term_painter::Attr::Plain;

macro_rules! print_diff {
    ([$colour:expr, $new:expr, $old:expr], $x:ident -> $a:expr, $b:expr) => ({
        let c = if $new.$x() { $a } else { $b };
        if $new.$x() != $old.$x() { $colour.with(|| { print!("{}", c); }); }
        else { print!("{}", c); }
    });
    ([$colour:expr, $new:expr, $old:expr], $x:ident) => (
        if $new.$x() != $old.$x() { $colour.with(|| { print!(" {}", $new.$x()); }); }
        else { print!(" {}", $new.$x()); }
    );
}

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
    pub fn print(&self) {
        let blue   = if self.colour { BrightBlue.to_style()   } else { Plain.to_style() };
        let yellow = if self.colour { BrightYellow.to_style() } else { Plain.to_style() };

        // Write PSRs.
        print!("{}\n\t\tCPSR: ", blue.paint("# Arm7Tdmi\n\t- Register Set"));
        Arm7TdmiDiff::print_psr(self.cpsr_old, self.cpsr_new, self.colour);
        print!("\tSPSR: ");
        if self.cpsr_new.mode() == Mode::User { print!("[---- -- ----- ---]"); }
        else { Arm7TdmiDiff::print_psr(self.spsr_old, self.spsr_new, self.colour); }
        println!("");

        // Write GPRs.
        for i in 0..16 {
            if (i % 4) == 0 { print!("\n\t\t"); }
            print!("{}[", Arm7TdmiDiff::DEBUG_REGISTER_NAMES[i]);
            if 0 != (self.gpr_new & (1 << i)) { yellow.with(|| { print!("{:08X}", self.gpr[i]); }); }
            else { print!("{:08X}", self.gpr[i]) }
            print!("]\t");
        }

        // Write pipeline state.
        print!("{}", blue.paint("\n\n\t- Pipeline State"));
        if self.cpsr_new.state() == State::ARM {
            println!("\n\t\tARM   Fetch:  {:#010X}\n\t\t\
                            ARM   Decode: {}\n", self.fetched_arm, self.decoded_arm);
        } else {
            println!("\n\t\tTHUMB Fetch:  {:#06X}\n\t\t\
                            THUMB Decode: {}\n", self.fetched_thumb, self.decoded_thumb);
        }
    }

    fn print_psr(old: PSR, new: PSR, colour: bool) {
        let yellow = if colour { BrightYellow.to_style() } else { Plain.to_style() };
        print!("[");
        print_diff!([yellow, new, old], N -> 'N', 'n');
        print_diff!([yellow, new, old], Z -> 'Z', 'z');
        print_diff!([yellow, new, old], C -> 'C', 'c');
        print_diff!([yellow, new, old], V -> 'V', 'v');
        print!(" ");
        print_diff!([yellow, new, old], irq_disabled -> 'I', 'i');
        print_diff!([yellow, new, old], fiq_disabled -> 'F', 'f');
        print_diff!([yellow, new, old], state);
        print_diff!([yellow, new, old], mode);
        print!("]");
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
