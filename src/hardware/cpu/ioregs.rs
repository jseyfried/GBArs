
use super::super::memory::IO_REGISTERS_LEN;
use super::super::memory::{RawBytes, Rom8, Rom16, Rom32, Ram8, Ram16, Ram32};


/// All memory-mapped GBA IO registers.
pub struct IoRegisters(Box<[u8; IO_REGISTERS_LEN as usize]>);

impl IoRegisters {
    /// Creates new zero initialised IO registers.
    pub fn new() -> IoRegisters {
        IoRegisters(box [0; IO_REGISTERS_LEN as usize])
    }
    
    /// Zero-fills all IO registers.
    pub fn clear(&mut self) {
        for i in 0..(IO_REGISTERS_LEN as usize) { (*self.0)[i] = 0 };
    }
}

impl RawBytes for IoRegisters {
    fn bytes<'a>(&'a self, offs: u32) -> &'a [u8] {
        &(*self.0)[(offs as usize)..]
    }
    
    fn bytes_mut<'a>(&'a mut self, offs: u32) -> &'a mut [u8] {
        &mut (*self.0)[(offs as usize)..]
    }
}

impl Rom8  for IoRegisters {}
impl Rom16 for IoRegisters {}
impl Rom32 for IoRegisters {}
impl Ram8  for IoRegisters {}
impl Ram16 for IoRegisters {}
impl Ram32 for IoRegisters {}
