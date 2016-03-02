// License below.
//! Implements the GamePak.
//!
//! A GamePak basically consists of two components:
//!
//! - A ROM chip.
//! - An SRAM chip.
//! - Optionally additional features.
//!
//! The ROM chip obviously contains the game itself.
//! This memory region is initialised by loading a
//! ROM file.
//!
//! The SRAM chip is where you game's progress will
//! be saved. The SRAM's contents will be dumped into
//! a saved game file.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use std::io;
use std::str;
use std::fmt;
use std::slice;
use std::io::Read;
use std::fs::File;
use std::path::Path;
use super::memory::GAME_PAK_WS0_ROM_LEN as GAME_PAK_ROM_LEN;
use super::memory::GAME_PAK_SRAM_LEN;
use super::memory::{RawBytes, Rom8, Rom16, Rom32, Ram8};


/// GBA ROMs are at most 32MiB in size.
pub const MAX_GBA_ROM_SIZE: usize = GAME_PAK_ROM_LEN as usize;

/// Offset of the ROM header's checksum.
pub const COMPLEMENT_CHECK_OFFSET: usize = 0xBD;

/// Offset of the game's title in ROM.
pub const GAME_TITLE_OFFSET: usize = 0xA0;

/// Maximum size of the game's title.
pub const GAME_TITLE_MAX_LEN: usize = 12;

/// Offset of the game's game code in ROM.
pub const GAME_CODE_OFFSET: usize = 0xAC;

/// Size of the game code.
pub const GAME_CODE_LEN: usize = 4;

/// Offset of the game's maker code in ROM.
pub const GAME_MAKER_CODE_OFFSET: usize = 0xB0;

/// Size of the maker code.
pub const GAME_MAKER_CODE_LEN: usize = 2;

/// Offset of the game's version number in ROM.
pub const GAME_VERSION_NUMBER: usize = 0xBC;



/// Helps making sense of the ROM's header bytes.
///
/// This is designed to separate handling meta data
/// from the task of just reading and writing binary
/// data to and from a ROM.
pub struct GamePakRomHeader<'a>(&'a GamePakRom);

impl<'a> GamePakRomHeader<'a> {
    /// Validates the ROM header by calculating a checksum.
    ///
    /// The checksum is calculated by subtracting all the bytes
    /// from `0xA0` to `0xBC` from zero and finally subtracting
    /// the magic number `0x19`. If the result equals the checksum
    /// byte at `0xBD`, the header is valid.
    ///
    /// # Returns
    /// - `true` if the checksums match.
    /// - `false` if the header checksum is invalid.
    pub fn complement_check(&self) -> bool {
        // Calculate checksum.
        let mut test = 0_u8;
        for i in 0xA0..0xBC {
            test = test.wrapping_sub(self.0.raw_bytes[i]);
        }
        test = test.wrapping_sub(0x19_u8);

        // Compare result.
        test == self.0.raw_bytes[COMPLEMENT_CHECK_OFFSET]
    }

    /// The currently loaded game's title.
    ///
    /// # Returns
    /// The title in up to 12 upper case ASCII letters.
    pub fn game_title(&'a self) -> &'a str {
        str::from_utf8(unsafe {
            slice::from_raw_parts(&(self.0.raw_bytes[GAME_TITLE_OFFSET]), self.0.loaded_rom_title_len)
        }).unwrap_or("????????????")
    }

    /// The currently loaded game's game code.
    ///
    /// # Returns
    /// A 4 letter upper case ASCII code.
    pub fn game_code(&'a self) -> &'a str {
        str::from_utf8(unsafe {
            slice::from_raw_parts(&(self.0.raw_bytes[GAME_CODE_OFFSET]), GAME_CODE_LEN)
        }).unwrap_or("????")
    }

    /// The currently loaded game's maker code.
    ///
    /// # Returns
    /// A 2 letter upper case ASCII code.
    /// `"01"` is Nintendo.
    pub fn game_maker_code(&'a self) -> &'a str {
        str::from_utf8(unsafe {
            slice::from_raw_parts(&(self.0.raw_bytes[GAME_MAKER_CODE_OFFSET]), GAME_MAKER_CODE_LEN)
        }).unwrap_or("??")
    }

    /// The currently loaded game's version number.
    ///
    /// # Returns
    /// Usually zero.
    pub fn game_version(&self) -> u8 {
        self.0.raw_bytes[GAME_VERSION_NUMBER]
    }

    /// The currently loaded ROM's size.
    ///
    /// # Returns
    /// Should be 1MiB aligned.
    /// At most 32MiB.
    pub fn rom_size(&self) -> usize {
        self.0.loaded_rom_len
    }
}

impl<'a> fmt::Display for GamePakRomHeader<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            formatter,
            "\"{title}\" v{version} [{code}:{maker}] ({len}MiB)",
            title   = self.game_title(),
            version = self.game_version(),
            code    = self.game_code(),
            maker   = self.game_maker_code(),
            len     = self.rom_size() / 1024 / 1024
        )
    }
}


/// Implements a GamePak' ROM chip.
pub struct GamePakRom {
    // Raw memory block. Nothing special here.
    raw_bytes: Box<[u8; MAX_GBA_ROM_SIZE]>,

    // Size of the currently loaded ROM.
    loaded_rom_len: usize,

    // Size of the loaded game's title.
    loaded_rom_title_len: usize,
}

impl GamePakRom {
    /// Creates a new ROM.
    ///
    /// All memory is initially zero-filled.
    pub fn new() -> GamePakRom {
        GamePakRom {
            // Some ROMs use 0xFF as unused memory.
            raw_bytes: box [0x00_u8; MAX_GBA_ROM_SIZE],

            loaded_rom_len: 0,

            loaded_rom_title_len: 0,
        }
    }

    /// Get a handle for the ROM's header.
    pub fn header<'a>(&'a self) -> GamePakRomHeader<'a> {
        GamePakRomHeader(self)
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
    pub fn load_from_file(&mut self, fp: &Path) -> io::Result<()> {
        // In case an error occurs and data is invalidated.
        self.loaded_rom_len = 0;
        self.loaded_rom_title_len = 0;

        // Loads a binary ROM from a given file and
        // fills the remaining space with zero bytes.
        trace!("Loading ROM file `{}`.", fp.display());
        let mut file = try!(File::open(fp));
        let rbytes = try!(file.read(&mut *self.raw_bytes));
        for i in rbytes..MAX_GBA_ROM_SIZE { self.raw_bytes[i] = 0 };
        self.loaded_rom_len = rbytes;

        // Decode the game's title's length without zero bytes.
        self.loaded_rom_title_len = GAME_TITLE_MAX_LEN;
        for i in 0..GAME_TITLE_MAX_LEN {
            if self.raw_bytes[GAME_TITLE_OFFSET + i] == 0 {
                self.loaded_rom_title_len = i;
                break;
            }
        }

        // Done.
        Ok(())
    }
}

impl RawBytes for GamePakRom {
    fn bytes<'a>(&'a self, offs: u32) -> &'a [u8] {
        &self.raw_bytes[(offs as usize)..]
    }

    fn bytes_mut<'a>(&'a mut self, offs: u32) -> &'a mut [u8] {
        &mut self.raw_bytes[(offs as usize)..]
    }
}

impl Rom8  for GamePakRom {}
impl Rom16 for GamePakRom {}
impl Rom32 for GamePakRom {}


/// Implements a GamePak's SRAM.
pub struct GamePakSram(Box<[u8; GAME_PAK_SRAM_LEN as usize]>);

impl GamePakSram {
    /// Creates a new zero-initialised SRAM.
    pub fn new() -> GamePakSram {
        GamePakSram(box [0; GAME_PAK_SRAM_LEN as usize])
    }

    /// Clears the SRAM.
    pub fn clear(&mut self) {
        for i in 0..(GAME_PAK_SRAM_LEN as usize) { (*self.0)[i] = 0 };
    }
}

impl RawBytes for GamePakSram {
    fn bytes<'a>(&'a self, offs: u32) -> &'a [u8] {
        &(*self.0)[(offs as usize)..]
    }

    fn bytes_mut<'a>(&'a mut self, offs: u32) -> &'a mut [u8] {
        &mut (*self.0)[(offs as usize)..]
    }
}

impl Rom8 for GamePakSram {}
impl Ram8 for GamePakSram {}


/// Implements a GamePak.
pub struct GamePak {
    rom: GamePakRom,
    sram: GamePakSram,
}

impl GamePak {
    /// Creates a new zero-initialised GamePak.
    pub fn new() -> GamePak {
        GamePak {
            rom: GamePakRom::new(),
            sram: GamePakSram::new(),
        }
    }

    /// Get the GamePak's ROM's header.
    pub fn header<'a>(&'a self) -> GamePakRomHeader<'a> {
        self.rom.header()
    }

    /// Get the GamePak's ROM.
    pub fn rom<'a>(&'a self) -> &'a GamePakRom {
        &self.rom
    }

    /// Get the GamePak's ROM.
    pub fn rom_mut<'a>(&'a mut self) -> &'a mut GamePakRom {
        &mut self.rom
    }

    /// Get the GamePak's SRAM.
    pub fn sram<'a>(&'a self) -> &'a GamePakSram {
        &self.sram
    }

    /// Get the GamePak's SRAM.
    pub fn sram_mut<'a>(&'a mut self) -> &'a mut GamePakSram {
        &mut self.sram
    }

    /// Loads a ROM from a file.
    ///
    /// Only ROMs up to 32MiB in size are valid.
    /// Everything beyond that size will be silently
    /// dropped.
    ///
    /// Unused memory is zero-filled and SRAM is cleared.
    ///
    /// # Params
    /// - `fp`: Path to the ROM file to load.
    ///
    /// # Returns
    /// - `Ok` if loaded successfully.
    /// - `Err` if an error occurred. The previous data might be damaged.
    pub fn load_rom_from_file(&mut self, fp: &Path) -> io::Result<()> {
        self.sram.clear();
        self.rom.load_from_file(fp)
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
