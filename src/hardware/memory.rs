// License below.
//! Utility types and constants for managing GBA memory.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use byteorder::{ByteOrder, LittleEndian};
use std::io;
use std::io::Read;
use std::fs::File;
use std::path::Path;


/// Address of the first byte of BIOS system ROM.
pub const BIOS_ROM_FIRST: u32 = 0x00000000;

/// Address of the last byte of BIOS system ROM.
pub const BIOS_ROM_LAST: u32 = 0x00003FFF;

/// Lenth of the BIOS system ROM area in bytes.
pub const BIOS_ROM_LEN: usize = ((BIOS_ROM_LAST+1) - BIOS_ROM_FIRST) as usize;

/// Address of the first byte of on-board WRAM.
pub const WRAM_ON_BOARD_FIRST: u32 = 0x02000000;

/// Address of the last byte of on-board WRAM.
pub const WRAM_ON_BOARD_LAST: u32 = 0x0203FFFF;

/// Length of the on-board WRAM area in bytes.
pub const WRAM_ON_BOARD_LEN: u32 = (WRAM_ON_BOARD_LAST+1) - WRAM_ON_BOARD_FIRST;

/// Address of the first byte of on-chip WRAM.
pub const WRAM_ON_CHIP_FIRST: u32 = 0x03000000;

/// Address of the last byte of on-chip WRAM.
pub const WRAM_ON_CHIP_LAST: u32 = 0x03007FFF;

/// Length of the on-chip WRAM area in bytes.
pub const WRAM_ON_CHIP_LEN: u32 = (WRAM_ON_CHIP_LAST+1) - WRAM_ON_CHIP_FIRST;

/// Address of the first byte of IO registers.
pub const IO_REGISTERS_FIRST: u32 = 0x04000000;

/// Address of the last byte of IO registers.
pub const IO_REGISTERS_LAST: u32 = 0x040003FE;

/// Length of the IO registers area in bytes.
pub const IO_REGISTERS_LEN: u32 = (IO_REGISTERS_LAST+1) - IO_REGISTERS_FIRST;

/// Address of the first byte of palette RAM.
pub const PALETTE_RAM_FIRST: u32 = 0x05000000;

/// Address of the last byte of palette RAM.
pub const PALETTE_RAM_LAST: u32 = 0x050003FF;

/// Length of the palette RAM area in bytes.
pub const PALETTE_RAM_LEN: u32 = (PALETTE_RAM_LAST+1) - PALETTE_RAM_FIRST;

/// Address of the first byte of VRAM.
pub const VRAM_FIRST: u32 = 0x06000000;

/// Address of the last byte of VRAM.
pub const VRAM_LAST: u32 = 0x06017FFF;

/// Length of the VRAM area in bytes.
pub const VRAM_LEN: u32 = (VRAM_LAST+1) - VRAM_FIRST;

/// Address of the first byte of OAM.
pub const OBJ_ATTRIBUTES_FIRST: u32 = 0x07000000;

/// Address of the last byte of OAM.
pub const OBJ_ATTRIBUTES_LAST: u32 = 0x070003FF;

/// Length of the OAM area in bytes.
pub const OBJ_ATTRIBUTES_LEN: u32 = (OBJ_ATTRIBUTES_LAST+1) - OBJ_ATTRIBUTES_FIRST;

/// Address of the first byte of Game Pak ROM in Wait State 0.
pub const GAME_PAK_WS0_ROM_FIRST: u32 = 0x08000000;

/// Address of the last byte of Game Pak ROM in Wait State 0.
pub const GAME_PAK_WS0_ROM_LAST: u32 = 0x09FFFFFF;

/// Length of the Game Pak ROM in Wait State 0 area in bytes.
pub const GAME_PAK_WS0_ROM_LEN: u32 = (GAME_PAK_WS0_ROM_LAST+1) - GAME_PAK_WS0_ROM_FIRST;

/// Address of the first byte of Game Pak ROM in Wait State 1.
pub const GAME_PAK_WS1_ROM_FIRST: u32 = 0x0A000000;

/// Address of the last byte of Game Pak ROM in Wait State 1.
pub const GAME_PAK_WS1_ROM_LAST: u32 = 0x0BFFFFFF;

/// Length of the Game Pak ROM in Wait State 1 area in bytes.
pub const GAME_PAK_WS1_ROM_LEN: u32 = (GAME_PAK_WS1_ROM_LAST+1) - GAME_PAK_WS1_ROM_FIRST;

/// Address of the first byte of Game Pak ROM in Wait State 2.
pub const GAME_PAK_WS2_ROM_FIRST: u32 = 0x0C000000;

/// Address of the last byte of Game Pak ROM in Wait State 2.
pub const GAME_PAK_WS2_ROM_LAST: u32 = 0x0DFFFFFF;

/// Length of the Game Pak ROM in Wait State 2 area in bytes.
pub const GAME_PAK_WS2_ROM_LEN: u32 = (GAME_PAK_WS2_ROM_LAST+1) - GAME_PAK_WS2_ROM_FIRST;

/// Address of the first byte of Game Pak SRAM.
pub const GAME_PAK_SRAM_FIRST: u32 = 0x0E000000;

/// Address of the last byte of Game Pak SRAM.
pub const GAME_PAK_SRAM_LAST: u32 = 0x0E00FFFF;

/// Length of the Game Pak SRAM area in bytes.
pub const GAME_PAK_SRAM_LEN: u32 = (GAME_PAK_SRAM_LAST+1) - GAME_PAK_SRAM_FIRST;


/// Maps global physical addresses to specialised local addresses.
///
/// The local addresses always start from 0.
pub enum PhysicalAddress {
    /// Address relative to the BIOS ROM area.
    BiosROM(u32),

    /// Address relative to the slower on-board WRAM.
    OnBoardWRAM(u32),

    /// Address relative to the quicker on-chip WRAM.
    OnChipWRAM(u32),

    /// Address relative to the IO registers area.
    RegistersIO(u32),

    /// Address relative to the palette RAM area.
    PaletteRAM(u32),

    /// Address relative to the VRAM area.
    VRAM(u32),

    /// Address relative to the object attributes area.
    AttributesOBJ(u32),

    /// Address relative to the GamePak ROM in wait state 0 area.
    GamePak0ROM(u32),

    /// Address relative to the GamePak ROM in wait state 1 area.
    GamePak1ROM(u32),

    /// Address relative to the GamePak ROM in wait state 2 area.
    GamePak2ROM(u32),

    /// Address relative to the GamePak SRAM area.
    GamePakSRAM(u32),

    /// An invalid physical address.
    Invalid(u32),
}

impl PhysicalAddress {
    /// Converts a global physical address to a local address.
    ///
    /// # Params
    /// - `p`: A 32-bit physical address.
    ///
    /// # Returns
    /// A mapped local address.
    pub fn from_u32(p: u32) -> PhysicalAddress {
        match p {
            BIOS_ROM_FIRST         ... BIOS_ROM_LAST         => PhysicalAddress::      BiosROM(p - BIOS_ROM_FIRST),
            WRAM_ON_BOARD_FIRST    ... WRAM_ON_BOARD_LAST    => PhysicalAddress::  OnBoardWRAM(p - WRAM_ON_BOARD_FIRST),
            WRAM_ON_CHIP_FIRST     ... WRAM_ON_CHIP_LAST     => PhysicalAddress::   OnChipWRAM(p - WRAM_ON_CHIP_FIRST),
            IO_REGISTERS_FIRST     ... IO_REGISTERS_LAST     => PhysicalAddress::  RegistersIO(p - IO_REGISTERS_FIRST),
            PALETTE_RAM_FIRST      ... PALETTE_RAM_LAST      => PhysicalAddress::   PaletteRAM(p - PALETTE_RAM_FIRST),
            VRAM_FIRST             ... VRAM_LAST             => PhysicalAddress::         VRAM(p - VRAM_FIRST),
            OBJ_ATTRIBUTES_FIRST   ... OBJ_ATTRIBUTES_LAST   => PhysicalAddress::AttributesOBJ(p - OBJ_ATTRIBUTES_FIRST),
            GAME_PAK_WS0_ROM_FIRST ... GAME_PAK_WS0_ROM_LAST => PhysicalAddress::  GamePak0ROM(p - GAME_PAK_WS0_ROM_FIRST),
            GAME_PAK_WS1_ROM_FIRST ... GAME_PAK_WS1_ROM_LAST => PhysicalAddress::  GamePak1ROM(p - GAME_PAK_WS1_ROM_FIRST),
            GAME_PAK_WS2_ROM_FIRST ... GAME_PAK_WS2_ROM_LAST => PhysicalAddress::  GamePak2ROM(p - GAME_PAK_WS2_ROM_FIRST),
            GAME_PAK_SRAM_FIRST    ... GAME_PAK_SRAM_LAST    => PhysicalAddress::  GamePakSRAM(p - GAME_PAK_SRAM_FIRST),
            _ => PhysicalAddress::Invalid(p),
        }
    }

    /// Converts a local address to a global physical address.
    ///
    /// # Params
    /// - `p`: A mapped local address.
    ///
    /// # Returns
    /// A global physical address.
    pub fn to_u32(&self) -> u32 {
        match *self {
            PhysicalAddress::BiosROM(p)       => p + BIOS_ROM_FIRST,
            PhysicalAddress::OnBoardWRAM(p)   => p + WRAM_ON_BOARD_FIRST,
            PhysicalAddress::OnChipWRAM(p)    => p + WRAM_ON_CHIP_FIRST,
            PhysicalAddress::RegistersIO(p)   => p + IO_REGISTERS_FIRST,
            PhysicalAddress::PaletteRAM(p)    => p + PALETTE_RAM_FIRST,
            PhysicalAddress::VRAM(p)          => p + VRAM_FIRST,
            PhysicalAddress::AttributesOBJ(p) => p + OBJ_ATTRIBUTES_FIRST,
            PhysicalAddress::GamePak0ROM(p)   => p + GAME_PAK_WS0_ROM_FIRST,
            PhysicalAddress::GamePak1ROM(p)   => p + GAME_PAK_WS1_ROM_FIRST,
            PhysicalAddress::GamePak2ROM(p)   => p + GAME_PAK_WS2_ROM_FIRST,
            PhysicalAddress::GamePakSRAM(p)   => p + GAME_PAK_SRAM_FIRST,
            PhysicalAddress::Invalid(p)       => p,
        }
    }
}


/// A trait for raw bytes memory.
pub trait RawBytes {
    /// Returns a byte slice starting at `offs`.
    ///
    /// # Params
    /// - `offs`: Lower bound of the slice.
    ///
    /// # Returns
    /// A slice of raw memory starting at `offs`
    /// and ending at whereever the memory area
    /// ends.
    ///
    /// # Panics
    /// Should panic if the given offset is out of bounds,
    /// as bounds checking should be done while converting
    /// global to local addresses.
    fn bytes(&self, offs: u32) -> &[u8];

    /// Returns a byte slice starting at `offs`.
    ///
    /// # Params
    /// - `offs`: Lower bound of the slice.
    ///
    /// # Returns
    /// A slice of raw memory starting at `offs`
    /// and ending at whereever the memory area
    /// ends.
    ///
    /// # Panics
    /// Should panic if the given offset is out of bounds,
    /// as bounds checking should be done while converting
    /// global to local addresses.
    fn bytes_mut(&mut self, offs: u32) -> &mut [u8];
}

/// A trait for 8-bit ROMs.
pub trait Rom8 : RawBytes {
    /// Reads a single byte from the ROM.
    ///
    /// # Params
    /// - `offs`: A ROM-local physical address.
    ///
    /// # Returns
    /// The data at the given offset.
    ///
    /// # Panics
    /// Should panic if the given offset is out of bounds,
    /// as bounds checking should be done while converting
    /// global to local addresses.
    fn read_byte(&self, offs: u32) -> u8 {
        self.bytes(offs)[0]
    }
}

/// A trait for 8-bit RAMs.
pub trait Ram8 : Rom8 {
    /// Writes a single byte to the RAM.
    ///
    /// # Params
    /// - `offs`: A ROM-local physical address.
    /// - `data`: The data to write.
    ///
    /// # Panics
    /// Should panic if the given offset is out of bounds,
    /// as bounds checking should be done while converting
    /// global to local addresses.
    fn write_byte(&mut self, offs: u32, data: u8) {
        self.bytes_mut(offs)[0] = data;
    }
}


/// A trait for 16-bit ROMs.
pub trait Rom16 : RawBytes {
    /// Reads two bytes from the ROM.
    ///
    /// # Params
    /// - `offs`: A ROM-local physical address. Bit 0 should be masked away.
    ///
    /// # Returns
    /// The data at the given offset.
    ///
    /// # Panics
    /// Should panic if the given offset is out of bounds,
    /// as bounds checking should be done while converting
    /// global to local addresses.
    fn read_halfword(&self, offs: u32) -> u16 {
        LittleEndian::read_u16( self.bytes(offs & !0b01) )
    }
}

/// A trait for 16-bit RAMs.
pub trait Ram16 : Rom16 {
    /// Writes two bytes to the RAM.
    ///
    /// # Params
    /// - `offs`: A ROM-local physical address. Bit 0 should be masked away.
    /// - `data`: The data to write.
    ///
    /// # Panics
    /// Should panic if the given offset is out of bounds,
    /// as bounds checking should be done while converting
    /// global to local addresses.
    fn write_halfword(&mut self, offs: u32, data: u16) {
        LittleEndian::write_u16( self.bytes_mut(offs & !0b01), data );
    }
}


/// A trait for 32-bit ROMs.
pub trait Rom32 : RawBytes {
    /// Reads four bytes from the ROM.
    ///
    /// # Params
    /// - `offs`: A ROM-local physical address. Bit 0 and 1 should be masked away.
    ///
    /// # Returns
    /// The data at the given offset.
    ///
    /// # Panics
    /// Should panic if the given offset is out of bounds,
    /// as bounds checking should be done while converting
    /// global to local addresses.
    fn read_word(&self, offs: u32) -> u32 {
        let w = LittleEndian::read_u32( self.bytes(offs & !0b11) );
        if (offs & 0b11) != 0 { w.rotate_right(8 * (offs & 0b11)); }
        w
    }
}

/// A trait for 32-bit RAMs.
pub trait Ram32 : Rom32 {
    /// Writes four bytes to the RAM.
    ///
    /// # Params
    /// - `offs`: A ROM-local physical address. Bit 0 and 1 should be masked away.
    /// - `data`: The data to write.
    ///
    /// # Panics
    /// Should panic if the given offset is out of bounds,
    /// as bounds checking should be done while converting
    /// global to local addresses.
    fn write_word(&mut self, offs: u32, data: u32) {
        LittleEndian::write_u32( self.bytes_mut(offs & !0b11), data );
    }
}


/// Implements the BIOS ROM area.
pub struct BiosRom(Box<[u8; BIOS_ROM_LEN]>);

impl BiosRom {
    /// Creates a new zero-filled BIOS ROM.
    pub fn new() -> BiosRom { BiosRom(box [0_u8; BIOS_ROM_LEN]) }

    /// Loads a ROM from a file.
    ///
    /// Only ROMs up to 1KiB in size are valid.
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
    pub fn load_from_file(&mut self, fp: &Path) -> io::Result<()> {
        // Loads a binary ROM from a given file and
        // fills the remaining space with zero bytes.
        trace!("Loading BIOS ROM file `{}`.", fp.display());
        let rbytes = try!(try!(File::open(fp)).read(&mut *self.0));
        for i in rbytes..BIOS_ROM_LEN { self.0[i] = 0 };
        Ok(())
    }
}

impl RawBytes for BiosRom {
    fn bytes(&self, offs: u32) -> &[u8] { &self.0[(offs as usize)..] }
    fn bytes_mut(&mut self, offs: u32) -> &mut [u8] { &mut self.0[(offs as usize)..] }
}
impl Rom8  for BiosRom {}
impl Rom16 for BiosRom {}
impl Rom32 for BiosRom {}

impl Default for BiosRom {
    fn default() -> BiosRom { BiosRom(box [0_u8; BIOS_ROM_LEN]) }
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
