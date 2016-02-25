
use std::path::Path;
use std::io;

use self::cpu::Arm7Tdmi;
pub use self::rom::*;


mod cpu;
pub mod rom;


pub struct Gba {
    //
    cpu: Arm7Tdmi,
    
    //
    rom: Rom,
}

impl Gba {
    //
    pub fn new() -> Gba {
        Gba {
            cpu: Arm7Tdmi::new(),
            rom: Rom::new(),
        }
    }
    
    //
    pub fn load_rom_from_file(&mut self, fp: &Path) -> io::Result<()> {
        self.rom.load_from_file(fp)
    }
    
    /// Get a handle for the ROM's header.
    ///
    /// This handle is used to query all kinds
    /// of meta data about the currently loaded
    /// ROM.
    ///
    /// # Returns
    /// A ROM header handle.
    pub fn rom_header<'a>(&'a self) -> RomHeader<'a> {
        self.rom.header()
    }
}
