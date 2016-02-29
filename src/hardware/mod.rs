
#![allow(dead_code)]

use std::path::Path;
use std::io;

pub use self::cpu::{Arm7Tdmi, ArmInstruction, IoRegisters};
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
        use self::memory::Rom32;
        let x = self.game_pak.load_rom_from_file(fp);
        debug!("First ROM instruction:\n{}", ArmInstruction::decode(self.game_pak.rom().read_word(0xC0+0x48) as i32).unwrap());
        x
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
