// License below.
//! Implements everything neccessary to emulate the GBA hardware.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use std::path::Path;
use std::io;

use self::cpu::Arm7Tdmi;
pub use self::error::*;
pub use self::gamepak::*;
pub use self::bus::*;


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
    bus: Bus,
}

impl Gba {
    /// Creates a new GBA emulator instance.
    pub fn new() -> Gba {
        Gba {
            cpu: Arm7Tdmi::new(),
            bus: Bus::new(),
        }
    }

    /// Loads a ROM from a file.
    ///
    /// Only ROMs up to 32MiB in size are valid.
    /// Everything beyond that size will be silently
    /// dropped.
    ///
    /// Unused memory is zero-filled.
    ///
    /// # Params
    /// - `fp`: Path to the ROM file to load.
    ///
    /// # Returns
    /// - `Ok` if loaded successfully.
    /// - `Err` if an error occurred. The previous data might be damaged.
    pub fn load_rom_from_file(&mut self, fp: &Path) -> io::Result<()> {
        self.bus.game_pak_mut().load_rom_from_file(fp)
    }

    /// Get a handle for the ROM's header.
    ///
    /// This handle is used to query all kinds
    /// of meta data about the currently loaded
    /// ROM.
    ///
    /// # Returns
    /// A ROM header handle.
    pub fn rom_header<'a>(&'a self) -> GamePakRomHeader<'a> {
        self.bus.game_pak().header()
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
