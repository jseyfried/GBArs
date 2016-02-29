
use std::error;
use std::fmt;


// TODO
#[derive(Debug)]
pub enum GbaError {
    InvalidArmInstruction(u32),
    InvalidThumbInstruction(u16),
    ReservedArmConditionNV,
}

impl error::Error for GbaError {
    fn description(&self) -> &str {
        match *self {
            GbaError::InvalidArmInstruction(_)   => "Invalid instruction in ARM state.",
            GbaError::InvalidThumbInstruction(_) => "Invalid instruction in THUMB state.",
            GbaError::ReservedArmConditionNV     => "Invalid NV condition in ARM state."
        }
    }
}

impl fmt::Display for GbaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GbaError::InvalidArmInstruction(x)   => write!(f, "Invalid ARM Instruction {:#08X}", x),
            GbaError::InvalidThumbInstruction(x) => write!(f, "Invalid THUMB Instruction {:#04X}", x),
            GbaError::ReservedArmConditionNV     => write!(f, "Invalid ARM condition NV"),
        }
    }
}
