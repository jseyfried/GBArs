// License below.
//! Implements emulation utilities for the GBA's main CPU, the ARM7TDMI.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use std::u32;
use std::cell::RefCell;
use std::rc::Rc;
use super::arminstruction::ArmInstruction;
use super::super::bus::*;

pub use self::exception::*;
pub use self::cpsr::*;
pub use self::armcondition::*;
pub use self::execarm::*;

pub mod exception;
pub mod cpsr;
pub mod armcondition;

mod execarm;

/// Implements the logic needed to emulate an ARM7TDMI CPU.
pub struct Arm7Tdmi {
    // Main register set.
    gpr: [i32; 16],
    cpsr: CPSR,
    spsr: [u32; 7],

    // Pipeline implementation.
    decoded: ArmInstruction,
    fetched: u32,

    // Register backups for mode changes.
    gpr_r8_r12_fiq: [i32; 5],
    gpr_r8_r12_other: [i32; 5],
    gpr_r13_all: [i32; 7],
    gpr_r14_all: [i32; 7],

    // Settings.
    mode: Mode,
    state: State,
    irq_disable: bool,
    fiq_disable: bool,

    // Connected devices.
    bus: Rc<RefCell<Bus>>,
}

impl Arm7Tdmi {
    /// Register index for the stack pointer.
    ///
    /// May be used as GPR in ARM state.
    pub const SP: usize = 13;

    /// Register index for the link register.
    ///
    /// This register usually holds the returns address
    /// of a running function. In ARM state, this might
    /// be used as GPR.
    pub const LR: usize = 14;

    /// Register index for the program counter.
    ///
    /// When reading PC, this will usually return an
    /// address beyond the read instruction's address,
    /// due to pipelining and other things.
    pub const PC: usize = 15;

    /// Creates a new CPU where all registers are zeroed.
    pub fn new(bus: Rc<RefCell<Bus>>) -> Arm7Tdmi {
        Arm7Tdmi {
            gpr: [0; 16],
            cpsr: CPSR(0),
            spsr: [0; 7],

            decoded: ArmInstruction::nop(),
            fetched: ArmInstruction::NOP_RAW,

            gpr_r8_r12_fiq: [0; 5],
            gpr_r8_r12_other: [0; 5],
            gpr_r13_all: [0; 7],
            gpr_r14_all: [0; 7],

            mode: Mode::System,
            state: State::ARM,
            irq_disable: false,
            fiq_disable: false,

            bus: bus,
        }
    }

    /// Resets the CPU.
    ///
    /// The CPU starts up by setting few
    /// register states and entering a
    /// reset exception.
    pub fn reset(&mut self) {
        self.gpr[Arm7Tdmi::PC] = 0;

        self.cpsr = CPSR(
            (CPSR::MODE_SUPERVISOR)
          | (1 << CPSR::IRQ_DISABLE_BIT)
          | (1 << CPSR::FIQ_DISABLE_BIT)
        );

        self.mode = Mode::Supervisor;
        self.state = State::ARM;
        self.irq_disable = true;
        self.fiq_disable = true;
    }

    /// Causes an exception, switching execution modes and states.
    pub fn exception(&mut self, ex: Exception) {
        self.change_mode(ex.mode_on_entry()); // Also sets LR.
        self.cpsr.set_state(State::ARM);
        self.state = State::ARM;
        self.cpsr.disable_irq();
        if ex.disable_fiq_on_entry() { self.cpsr.disable_fiq(); }
        // TODO LR = PC + whatevs
        self.gpr[Arm7Tdmi::PC] = ex.vector_address() as i32;
    }

    fn change_mode(&mut self, new_mode: Mode) {
        let cmi = self.mode as u8 as usize;
        let nmi =  new_mode as u8 as usize;

        // Save banked registers R13, R14, SPSR.
        let ret_addr = self.gpr[Arm7Tdmi::PC] + 0; // TODO special offset by exception type
        self.gpr_r14_all[cmi] = self.gpr[14];
        self.gpr_r14_all[nmi] = ret_addr;
        self.gpr[14] = ret_addr;
        self.gpr_r13_all[cmi] = self.gpr[13];
        self.gpr[13] = self.gpr_r13_all[nmi];
        self.spsr[nmi] = self.cpsr.0;

        // Now the banked registers R8..R12.
        if (new_mode == Mode::FIQ) ^ (self.mode == Mode::FIQ) {
            if new_mode == Mode::FIQ {
                for i in 0..5 { self.gpr_r8_r12_other[i] = self.gpr[i+8]; }
                for i in 0..5 { self.gpr[i+8] = self.gpr_r8_r12_fiq[i]; }
            }
            else {
                for i in 0..5 { self.gpr_r8_r12_fiq[i] = self.gpr[i+8]; }
                for i in 0..5 { self.gpr[i+8] = self.gpr_r8_r12_other[i]; }
            }
        }

        // Apply new state.
        self.cpsr.set_mode(new_mode);
        self.mode = new_mode;
    }

    fn clear_pipeline(&mut self) {
        self.decoded = ArmInstruction::nop();
        self.fetched = ArmInstruction::NOP_RAW;
    }

    fn update_flags(&mut self, x: i32, y: u64) {
        self.cpsr.set_N(x  < 0);
        self.cpsr.set_Z(x == 0);
        self.cpsr.set_C((y & 0x1_00000000_u64) != 0);
        self.cpsr.set_V( y > (u32::MAX as u64));
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
