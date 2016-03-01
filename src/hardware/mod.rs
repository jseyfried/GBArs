// License below.
//! Implements everything neccessary to emulate the GBA hardware.
#![allow(dead_code)]
#![warn(missing_docs)]

use std::path::Path;
use std::io;

use self::cpu::{Arm7Tdmi, IoRegisters};
pub use self::error::*;
pub use self::gamepak::*;


pub mod cpu;
pub mod memory;
pub mod gamepak;
pub mod error;

pub struct Gba {
    //
    cpu: Arm7Tdmi,
    
    //
    ioregs: IoRegisters,
    
    //
    game_pak: GamePak,
}

impl Gba {
    //
    pub fn new() -> Gba {
        Gba {
            cpu: Arm7Tdmi::new(),
            ioregs: IoRegisters::new(),
            game_pak: GamePak::new(),
        }
    }
    
    //
    pub fn load_rom_from_file(&mut self, fp: &Path) -> io::Result<()> {
        self.game_pak.load_rom_from_file(fp)
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
        self.game_pak.header()
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
