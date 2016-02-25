
#![allow(dead_code)]

use std::path::Path;
use std::io;

use self::cpu::Arm7Tdmi;
pub use self::gamepak::*;


mod cpu;
mod memory;
pub mod gamepak;


pub struct Gba {
    //
    cpu: Arm7Tdmi,
    
    //
    game_pak: GamePak,
}

impl Gba {
    //
    pub fn new() -> Gba {
        Gba {
            cpu: Arm7Tdmi::new(),
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
