

use std::mem;


/// The condition field of an ARM instruction.
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum ArmCondition {
    /// Z set. EQual.
    EQ = 0b0000,
    
    /// Z clear. Not Equal.
    NE = 0b0001,
    
    /// C set. Unsigned Higher or Same.
    HS = 0b0010,
    
    /// C clear. Unsigned LOwer.
    LO = 0b0011,
    
    /// N set. MInus, i.e. negative.
    MI = 0b0100,
    
    /// N clear. PLus, i.e. positive or zero.
    PL = 0b0101,
    
    /// V Set. Overflow.
    VS = 0b0110,
    
    /// V Clear. No overflow.
    VC = 0b0111,
    
    /// C set and Z clear. Unsigned HIgher.
    HI = 0b1000,
    
    /// C clear or Z set. Unsigned Lower or Same.
    LS = 0b1001,
    
    /// N equals V. Greater than or Equal to.
    GE = 0b1010,
    
    /// N distinct from V. Less Than.
    LT = 0b1011,
    
    /// Z clear and N equals V. Greater Than.
    GT = 0b1100,
    
    /// Z set or N distinct from V. Less than or Equal to.
    LE = 0b1101,
    
    /// ALways execute this instruction, i.e. no condition.
    AL = 0b1110,
    
    /// Reserved.
    NV = 0b1111,
}


// TODO
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ArmOpcode {
    Invalid,
    
    // Branches.
    B,  // PC += offs
    BL, // LR = PC-4, PC += offs
    BX, // PC +=
    
    // Arithmetic.
    ADD, // D = Op1 + Op2
    ADC, // D = Op1 + Op2 + carry
    SUB, // D = Op1 - Op2
    SBC, // D = Op1 - Op2 + carry - 1
    RSB, // D = Op2 - Op1
    RSC, // D = Op2 - Op1 + carry - 1
    
    // Comparisons.
    CMP, // Op1 - Op2, only flags set
    CMN, // Op1 + Op2, only flags set
    TST, // Op1 & Op2, only flags set
    TEQ, // Op1 ^ Op2, only flags set
    
    // Logical Operations.
    AND, // D = Op1 & Op2
    EOR, // D = Op1 ^ Op2
    ORR, // D = Op1 | Op2
    BIC, // D = Op1 & !Op2, i.e. bit clear
    
    // Data Movement.
    MOV, // D = Op2
    MVN, // D = !Op2
    
    // Multiplication.
    MUL,  // Rd = Rm * Rs
    MLA,  // Rd = (Rm * Rs) + Rn
    MULL, // RdHi_RdLo = Rm * Rs
    MLAL, // RdHi_RdLo = (Rm * Rs) + RdHi_RdLo
    
    // Load/Store Instructions.
    LDR,  // Load word.
    STR,  // Store word.
    LDRH, // Load signed/unsigned halfword.
    STRH, // Store signed/unsigned halfword.
    LDRB, // Load signed/unsigned byte.
    STRB, // Store signed/unsigned byte.
    
    // Block data transfer.
    LDM,  // Load multiple words.
    STM,  // Store multiple words.
    
    // PSR transfer.
    MRS,
    MSR,
}

impl ArmOpcode {
    /// Decodes an ARM data processing opcode from a 4-bit number.
    ///
    /// # Params
    /// - `b`: A 4-bit opcode. Upper half will be ignored.
    ///
    /// # Returns
    /// A data processing opcode.
    pub fn data_processing_from_bits(b: u8) -> ArmOpcode {
        match b & 0b1111 {
            0b0000 => ArmOpcode::AND,
            0b0001 => ArmOpcode::EOR,
            0b0010 => ArmOpcode::SUB,
            0b0011 => ArmOpcode::RSB,
            0b0100 => ArmOpcode::ADD,
            0b0101 => ArmOpcode::ADC,
            0b0110 => ArmOpcode::SBC,
            0b0111 => ArmOpcode::RSC,
            0b1000 => ArmOpcode::TST,
            0b1001 => ArmOpcode::TEQ,
            0b1010 => ArmOpcode::CMP,
            0b1011 => ArmOpcode::CMN,
            0b1100 => ArmOpcode::ORR,
            0b1101 => ArmOpcode::MOV,
            0b1110 => ArmOpcode::BIC,
            0b1111 => ArmOpcode::MVN,
            _ => unreachable!(),
        }
    }
}


// TODO
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ArmBarrelShifterOp {
    LSL(u8), // Logical Shift Left, CF << dst << 0
    LSR(u8), // Logical Shift Right, 0 >> dst >> CF
    ASR(u8), // Arithmetic Shift Right, dst#31 >> dst >> CF
    ROR(u8), // ROtate Right, into CF
}

impl ArmBarrelShifterOp {
    /// Decodes a shift operation from a 2-bit opcode.
    ///
    /// # Params
    /// - `b`: 2-bit opcode. Upper bits will be ignored.
    /// - `sh`: The shift to apply.
    ///
    /// # Returns
    /// A barrel shifter operation.
    pub fn from_bits(b: u8, sh: u8) -> ArmBarrelShifterOp {
        match b & 0b11 {
            0 => ArmBarrelShifterOp::LSL(sh),
            1 => ArmBarrelShifterOp::LSR(sh),
            2 => ArmBarrelShifterOp::ASR(sh),
            3 => ArmBarrelShifterOp::ROR(sh),
            _ => unreachable!(),
        }
    }
    
    /// Performs this shift operation on the given value.
    pub fn execute_no_carry(self, x: i32) -> i32 {
        match self {
            ArmBarrelShifterOp::LSL(sh) => x.wrapping_shl(sh as u32),
            ArmBarrelShifterOp::LSR(sh) => (x as u32).wrapping_shr(sh as u32) as i32,
            ArmBarrelShifterOp::ASR(sh) => x.wrapping_shr(sh as u32),
            ArmBarrelShifterOp::ROR(sh) => x.rotate_right((sh as u32) & 0x1F),
        }
    }
}


// TODO
#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(non_snake_case)]
pub struct ArmInstruction {
    immediate: i32,
    cond: ArmCondition,
    opcode: ArmOpcode,
    shift_op: ArmBarrelShifterOp,
    Rd: u8,
    Rn: u8,
    Rs: u8,
    Rm: u8,
    has_immediate: bool,
    shift_from_reg: bool,
    signed: bool,
    set_flags: bool,
    post_indexed: bool,
    auto_increment: bool,
    force_usermode: bool,
    spsr: bool,
}

impl ArmInstruction {
    /// Creates an invalid instruction.
    pub fn new() -> ArmInstruction {
        ArmInstruction {
            immediate:      0,
            cond:           ArmCondition::AL,
            opcode:         ArmOpcode::Invalid,
            shift_op:       ArmBarrelShifterOp::LSL(0),
            Rd:             0,
            Rn:             0,
            Rs:             0,
            Rm:             0,
            has_immediate:  false,
            shift_from_reg: false,
            signed:         false,
            set_flags:      false,
            post_indexed:   false,
            auto_increment: false,
            force_usermode: false,
            spsr:           false,
        }
    }
    
    /// Takes a raw 32-bit instruction and decodes
    /// it into something easier to interpret by
    /// software.
    ///
    /// # Params
    /// - `raw`: The raw 32-bit instruction.
    ///
    /// # Returns
    /// A decoded instruction.
    pub fn decode(raw: u32) -> ArmInstruction {
        let mut tmp = ArmInstruction::new();
        tmp.cond = unsafe { mem::transmute( ((raw >> 28) & 0b1111) as u8 ) };
        
        // Branch?
        if (raw & 0x0E000000_u32) == 0x0A000000_u32 {
            tmp.immediate = ((raw << 8) as i32) >> 6; // LSL by 2 bits and sign extension.
            tmp.has_immediate = true;
            tmp.opcode = if (raw & 0x01000000_u32) == 0 { ArmOpcode::B } else { ArmOpcode::BL };
        }
        // Branch exchange?
        else if (raw & 0x0FFFFFF0_u32) == 0x012FFF10_u32 {
            tmp.Rn     = (raw & 0x0F) as u8;
            tmp.opcode = ArmOpcode::BX;
        }
        // Data processing?
        else if (raw & 0x0C000000_u32) == 0 {
            // Way too complex for one function.
            tmp.decode_data_processing(raw);
        }
        // MRS instruction?
        else if (raw & 0x0FBF0FFF_u32) == 0x010F0000_u32 {
            tmp.spsr   = (raw & (1 << 22)) != 0;
            tmp.Rd     = ((raw >> 12) & 0b1111) as u8;
            tmp.opcode = ArmOpcode::MRS;
        }
        // MSR instruction?
        else if (raw & 0x0FBEFFF0_u32) == 0x0128F000_u32 {
            tmp.spsr   = (raw & (1 << 22)) != 0;
            tmp.Rm     = (raw & 0b1111) as u8;
            tmp.opcode = ArmOpcode::MSR;
        }
        else if (raw & 0x0FBFF000_u32) == 0x0328F000_u32 {
            let bits          = ((raw >> 8) & 0b1111) * 2;
            tmp.has_immediate = true;
            tmp.immediate     = (raw & 0xFF).rotate_right(bits) as i32;
            tmp.spsr          = (raw & (1 << 22)) != 0;
            tmp.opcode        = ArmOpcode::MSR;
        }
        // Multiply or Multiply Accumulate? Long?
        else if (raw & 0x0F0000F0_u32) == 0x00000090_u32 {
            // Again quite a bit of code.
            // And a possible bug, as MLA/MUL instructions
            // with a set sign bit will be accepted here.
            tmp.decode_multiplication(raw);
        }
        // Single data transfer?
        else if (raw & 0x1_u32) == 0x0_u32 {
            //
            unimplemented!();
        }
        // Undefined or unimplemented opcode.
        else {
            tmp.immediate = raw as i32;
            tmp.opcode    = ArmOpcode::Invalid;
            error!("Decoding invalid or unimplemented instruction: {:#8X}", raw);
        }
        
        // Done decoding.
        tmp
    }
    
    
    fn decode_data_processing(&mut self, raw: u32) {
        // Decode the obvious parameters.
        self.has_immediate = (raw & (1 << 25)) != 0;
        self.set_flags     = (raw & (1 << 20)) != 0;
        self.Rn = ((raw >> 16) & 0b1111) as u8;
        self.Rd = ((raw >> 12) & 0b1111) as u8;
        
        // Decode shifting.
        if self.has_immediate {
            let bits = ((raw >> 8) & 0b1111) * 2;
            self.immediate = (raw & 0xFF).rotate_right(bits) as i32;
        } else {
            self.Rm = (raw & 0b1111) as u8;
            let sh: u8 = if (raw & 0x10) == 0 {
                ((raw >> 7) & 0x1F) as u8
            } else {
                self.Rs = ((raw >> 8) & 0b1111) as u8;
                self.shift_from_reg = true;
                0_u8
            };
            self.shift_op = ArmBarrelShifterOp::from_bits((raw >> 5) as u8, sh);
        }
        
        // Decode the opcode.
        self.opcode = ArmOpcode::data_processing_from_bits((raw >> 21) as u8);
    }
    
    fn decode_multiplication(&mut self, raw: u32) {
        // Operand registers are the same.
        self.Rd = ((raw >> 16) & 0b1111) as u8;
        self.Rn = ((raw >> 12) & 0b1111) as u8;
        self.Rs = ((raw >>  8) & 0b1111) as u8;
        self.Rm = ((raw >>  0) & 0b1111) as u8;
        
        // This trick exploits instruction similarities
        // to reduce the amount of code. Usually, a MUL
        // or MLA instruction must not have a set sign bit.
        // However, this code here does accept a set one.
        self.signed    = (raw & (1 << 22)) != 0;
        self.set_flags = (raw & (1 << 20)) != 0;
        let accum      = (raw & (1 << 21)) != 0;
        self.opcode = if (raw & (1 << 23)) != 0 {
            if accum { ArmOpcode::MLAL } else { ArmOpcode::MULL }
        } else {
            if self.signed { warn!("MLA/MUL sign bit should be zero.") };
            if accum { ArmOpcode::MLA } else { ArmOpcode::MUL }
        };
    }
}
