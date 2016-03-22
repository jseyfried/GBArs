// License below.
//! Implements everything neccessary to emulate the GBA hardware.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use std::cell::{RefCell, Ref, RefMut};
use std::rc::Rc;

use self::cpu::Arm7Tdmi;
use self::bus::*;
pub use self::error::*;
pub use self::gamepak::*;


pub mod cpu;
pub mod memory;
pub mod gamepak;
pub mod error;
pub mod ioregs;
pub mod bus;


/// This is the actual GBA emulator. It handles all the virtual hardware,
/// loads and saves ROMs and SRAMs, executes the CPU instructions, and
/// what not.
pub struct Gba {
    cpu: Arm7Tdmi,
    bios: Rc<RefCell<memory::BiosRom>>,
    game_pak: Rc<RefCell<GamePak>>,
}

impl Gba {
    /// Creates a new GBA emulator instance.
    pub fn new() -> Gba {
        let bios = Rc::new(RefCell::new(memory::BiosRom::new()));
        let gpak = Rc::new(RefCell::new(GamePak::new()));
        let bus = Rc::new(RefCell::new(Bus::new(gpak.clone(), bios.clone())));
        Gba {
            cpu: Arm7Tdmi::new(bus.clone()),
            bios: bios,
            game_pak: gpak,
        }
    }

    /// Get an immutable reference to the GamePak.
    pub fn game_pak(&self) -> Ref<GamePak> { self.game_pak.borrow() }

    /// Get a mutable reference to the GamePak.
    pub fn game_pak_mut(&mut self) -> RefMut<GamePak> { self.game_pak.borrow_mut() }

    /// Get an immutable reference to the BIOS ROM.
    pub fn bios(&self) -> Ref<memory::BiosRom> { self.bios.borrow() }

    /// Get a mutable reference to the BIOS ROM.
    pub fn bios_mut(&mut self) -> RefMut<memory::BiosRom> { self.bios.borrow_mut() }

    /// Get an immmutable reference to the ARM7TDMI CPU emulator.
    pub fn cpu_arm7tdmi<'a>(&'a self) -> &'a Arm7Tdmi { &self.cpu }

    /// Get a mutable reference to the ARM7TDMI CPU emulator.
    pub fn cpu_arm7tdmi_mut<'a>(&'a mut self) -> &'a mut Arm7Tdmi { &mut self.cpu }
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
