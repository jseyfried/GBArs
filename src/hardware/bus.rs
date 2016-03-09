// License below.
//! Provides utilities for emulating the GBA's memory/bus system.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use super::memory::*;
use super::gamepak::*;
use super::ioregs::*;
use super::error::*;

// TODO how to handle aborts?
/// Implements the memory and bus system of the GBA.
pub struct Bus {
    ioregs: IoRegisters,
    game_pak: GamePak,
}

impl Bus {
    /// Creates a new memory and bus system object.
    pub fn new() -> Bus {
        Bus {
            ioregs: IoRegisters::new(),
            game_pak: GamePak::new(),
        }
    }

    /// Get a reference to the connected GamePak.
    pub fn game_pak<'a>(&'a self) -> &'a GamePak { &self.game_pak }

    /// Get a mutable reference to the connected GamePak.
    pub fn game_pak_mut<'a>(&'a mut self) -> &'a mut GamePak { &mut self.game_pak }

    /// Loads a word from the memory system.
    ///
    /// The given address will be rounded down to the next word-aligned
    /// address. This new address will be used to load the desired word.
    /// Then, the loaded word will be rotated right by as many bytes as
    /// the initial address was missaligned.
    ///
    /// Imagine you load a word from the address `0x0012`, which is
    /// missaligned by two bytes. First, the word at `0x0010` will
    /// be loaded. Then, this word will be rotated right by 2 bytes,
    /// i.e. 16 bits, as the address was missaligned by two.
    ///
    /// ## Params
    /// - `addr`: The address to load the word from.
    ///
    /// ## Returns
    /// - `Ok`: The loaded word.
    /// - `Err(InvalidPhysicalAddress)`: The given address is not part of the memory map.
    /// - `Err(InvalidMemoryBusWidth)`: The memory-mapped device cannot load words.
    pub fn load_word(&self, addr: u32) -> Result<i32, GbaError> {
        match PhysicalAddress::from_u32(addr) {
            PhysicalAddress::BiosROM(p)       => unimplemented!(),
            PhysicalAddress::OnBoardWRAM(p)   => unimplemented!(),
            PhysicalAddress::OnChipWRAM(p)    => unimplemented!(),
            PhysicalAddress::RegistersIO(p)   => Ok(self.ioregs.read_word(p) as i32),
            PhysicalAddress::PaletteRAM(p)    => unimplemented!(),
            PhysicalAddress::VRAM(p)          => unimplemented!(),
            PhysicalAddress::AttributesOBJ(p) => unimplemented!(),
            PhysicalAddress::GamePak0ROM(p)   => unimplemented!(),
            PhysicalAddress::GamePak1ROM(p)   => unimplemented!(),
            PhysicalAddress::GamePak2ROM(p)   => unimplemented!(),
            PhysicalAddress::GamePakSRAM(p)   => unimplemented!(),
            PhysicalAddress::Invalid(p)       => Err(GbaError::InvalidPhysicalAddress(p)),
        }
    }

    /// Stores a word in the memory system.
    ///
    /// Any given address will be rounded down to the next
    /// word-aligned address.
    ///
    /// ## Params
    /// - `addr`: The address to store the word to.
    /// - `data`: The word to store.
    ///
    /// ## Returns
    /// - `Ok`: Storing succeeded.
    /// - `Err(InvalidPhysicalAddress)`: The given address is not part of the memory map.
    /// - `Err(InvalidMemoryBusWidth)`: The memory-mapped device cannot store words.
    /// - `Err(InvalidRomAccess)`: Tried storing data into a ROM.
    pub fn store_word(&mut self, addr: u32, data: i32) -> Result<(), GbaError> {
        match PhysicalAddress::from_u32(addr) {
            PhysicalAddress::BiosROM(p)       => unimplemented!(),
            PhysicalAddress::OnBoardWRAM(p)   => unimplemented!(),
            PhysicalAddress::OnChipWRAM(p)    => unimplemented!(),
            PhysicalAddress::RegistersIO(p)   => Ok(self.ioregs.write_word(p, data as u32)),
            PhysicalAddress::PaletteRAM(p)    => unimplemented!(),
            PhysicalAddress::VRAM(p)          => unimplemented!(),
            PhysicalAddress::AttributesOBJ(p) => unimplemented!(),
            PhysicalAddress::GamePak0ROM(p)   => unimplemented!(),
            PhysicalAddress::GamePak1ROM(p)   => unimplemented!(),
            PhysicalAddress::GamePak2ROM(p)   => unimplemented!(),
            PhysicalAddress::GamePakSRAM(p)   => unimplemented!(),
            PhysicalAddress::Invalid(p)       => Err(GbaError::InvalidPhysicalAddress(p)),
        }
    }

    /// Loads a byte from the memory system.
    ///
    /// ## Params
    /// - `addr`: The address to load the byte from.
    ///
    /// ## Returns
    /// - `Ok`: The loaded byte.
    /// - `Err(InvalidPhysicalAddress)`: The given address is not part of the memory map.
    /// - `Err(InvalidMemoryBusWidth)`: The memory-mapped device cannot load bytes.
    pub fn load_byte(&self, addr: u32) -> Result<i32, GbaError> {
        match PhysicalAddress::from_u32(addr) {
            PhysicalAddress::BiosROM(p)       => unimplemented!(),
            PhysicalAddress::OnBoardWRAM(p)   => unimplemented!(),
            PhysicalAddress::OnChipWRAM(p)    => unimplemented!(),
            PhysicalAddress::RegistersIO(p)   => Ok(self.ioregs.read_byte(p) as u32 as i32),
            PhysicalAddress::PaletteRAM(p)    => unimplemented!(),
            PhysicalAddress::VRAM(p)          => unimplemented!(),
            PhysicalAddress::AttributesOBJ(p) => unimplemented!(),
            PhysicalAddress::GamePak0ROM(p)   => unimplemented!(),
            PhysicalAddress::GamePak1ROM(p)   => unimplemented!(),
            PhysicalAddress::GamePak2ROM(p)   => unimplemented!(),
            PhysicalAddress::GamePakSRAM(p)   => unimplemented!(),
            PhysicalAddress::Invalid(p)       => Err(GbaError::InvalidPhysicalAddress(p)),
        }
    }

    /// Stores a byte in the memory system.
    ///
    /// ## Params
    /// - `addr`: The address to store the byte to.
    /// - `data`: The byte to store.
    ///
    /// ## Returns
    /// - `Ok`: Storing succeeded.
    /// - `Err(InvalidPhysicalAddress)`: The given address is not part of the memory map.
    /// - `Err(InvalidMemoryBusWidth)`: The memory-mapped device cannot store bytes.
    /// - `Err(InvalidRomAccess)`: Tried storing data into a ROM.
    pub fn store_byte(&mut self, addr: u32, data: i32) -> Result<(), GbaError> {
        match PhysicalAddress::from_u32(addr) {
            PhysicalAddress::BiosROM(p)       => unimplemented!(),
            PhysicalAddress::OnBoardWRAM(p)   => unimplemented!(),
            PhysicalAddress::OnChipWRAM(p)    => unimplemented!(),
            PhysicalAddress::RegistersIO(p)   => Ok(self.ioregs.write_byte(p, (data & 0xFF) as u8)),
            PhysicalAddress::PaletteRAM(p)    => unimplemented!(),
            PhysicalAddress::VRAM(p)          => unimplemented!(),
            PhysicalAddress::AttributesOBJ(p) => unimplemented!(),
            PhysicalAddress::GamePak0ROM(p)   => unimplemented!(),
            PhysicalAddress::GamePak1ROM(p)   => unimplemented!(),
            PhysicalAddress::GamePak2ROM(p)   => unimplemented!(),
            PhysicalAddress::GamePakSRAM(p)   => unimplemented!(),
            PhysicalAddress::Invalid(p)       => Err(GbaError::InvalidPhysicalAddress(p)),
        }
    }

    /// Loads a halfword from the memory system.
    ///
    /// The result of a missaligned load is undefined.
    ///
    /// ## Params
    /// - `addr`: The address to load the halfword from.
    ///
    /// ## Returns
    /// - `Ok`: The loaded halfword.
    /// - `Err(InvalidPhysicalAddress)`: The given address is not part of the memory map.
    /// - `Err(InvalidMemoryBusWidth)`: The memory-mapped device cannot load halfwords.
    pub fn load_halfword(&self, addr: u32) -> Result<i32, GbaError> {
        if 0 != (addr & 0b01) { warn!("Reading missaligned halfword address {:#010X}.", addr); }
        match PhysicalAddress::from_u32(addr) {
            PhysicalAddress::BiosROM(p)       => unimplemented!(),
            PhysicalAddress::OnBoardWRAM(p)   => unimplemented!(),
            PhysicalAddress::OnChipWRAM(p)    => unimplemented!(),
            PhysicalAddress::RegistersIO(p)   => Ok(self.ioregs.read_halfword(p) as u32 as i32),
            PhysicalAddress::PaletteRAM(p)    => unimplemented!(),
            PhysicalAddress::VRAM(p)          => unimplemented!(),
            PhysicalAddress::AttributesOBJ(p) => unimplemented!(),
            PhysicalAddress::GamePak0ROM(p)   => unimplemented!(),
            PhysicalAddress::GamePak1ROM(p)   => unimplemented!(),
            PhysicalAddress::GamePak2ROM(p)   => unimplemented!(),
            PhysicalAddress::GamePakSRAM(p)   => unimplemented!(),
            PhysicalAddress::Invalid(p)       => Err(GbaError::InvalidPhysicalAddress(p)),
        }
    }

    /// Stores a halfword in the memory system.
    ///
    /// The result of a missaligned store is undefined.
    ///
    /// ## Params
    /// - `addr`: The address to store the halfword to.
    /// - `data`: The halfword to store.
    ///
    /// ## Returns
    /// - `Ok`: Storing succeeded.
    /// - `Err(InvalidPhysicalAddress)`: The given address is not part of the memory map.
    /// - `Err(InvalidMemoryBusWidth)`: The memory-mapped device cannot store halfwords.
    /// - `Err(InvalidRomAccess)`: Tried storing data into a ROM.
    pub fn store_halfword(&mut self, addr: u32, data: i32) -> Result<(), GbaError> {
        if 0 != (addr & 0b01) { warn!("Reading missaligned halfword address {:#010X}.", addr); }
        match PhysicalAddress::from_u32(addr) {
            PhysicalAddress::BiosROM(p)       => unimplemented!(),
            PhysicalAddress::OnBoardWRAM(p)   => unimplemented!(),
            PhysicalAddress::OnChipWRAM(p)    => unimplemented!(),
            PhysicalAddress::RegistersIO(p)   => Ok(self.ioregs.write_halfword(p, (data & 0xFFFF) as u16)),
            PhysicalAddress::PaletteRAM(p)    => unimplemented!(),
            PhysicalAddress::VRAM(p)          => unimplemented!(),
            PhysicalAddress::AttributesOBJ(p) => unimplemented!(),
            PhysicalAddress::GamePak0ROM(p)   => unimplemented!(),
            PhysicalAddress::GamePak1ROM(p)   => unimplemented!(),
            PhysicalAddress::GamePak2ROM(p)   => unimplemented!(),
            PhysicalAddress::GamePakSRAM(p)   => unimplemented!(),
            PhysicalAddress::Invalid(p)       => Err(GbaError::InvalidPhysicalAddress(p)),
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
