

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
    LDRW, // Load word.
    STRW, // Store word.
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
    
    // Swap.
    SWPW,
    SWPB,
    
    // Software Interrupt
    SWI,
    
    // Co-Processor stuff.
    CDP(u8), // Co-processor Data oPerations with opcode (u8).
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


/*
    Instruction Flags:
        .... ....  .... ....  .... ....  .... ....
        COND                                  RegM | BX #map: RegN => RegM
        COND    F  imm_ imm_  imm_ imm_  imm_ imm_ | B/BL
        COND   Ix  xxxS RegN  RegD shft  shft shft | Data Processing Op4(xxxx)...
               0                   _imm  _op0 RegM | ... RegM = RegM SHIFT(op) _imm_
               0                   RegS  0op1 RegM | ... RegM = RegM SHIFT(op) RegS
               1                   xxxx  imm_ imm_ | ... Immediate = imm_imm_ ROR 2*xxxx
        COND        P         RegD                 | MRS
        COND        P                         RegM | MSR by RegM
        COND   I    P              shft  shft shft | MSR by Immediate...
               0                   _imm  _op0 RegM | ... RegM = RegM SHIFT(op) _imm_
               0                   RegS  0op1 RegM | ... RegM = RegM SHIFT(op) RegS
               1                   xxxx  imm_ imm_ | ... Immediate = imm_imm_ ROR 2*xxxx
        COND         AS RegN  RegD RegS       RegM | MUL/MLA #map: RegN <=> RegD
        COND        UAS RegN  RegD RegS       RegM | MULL/MLAL #map: RdHi => RegN, RdLo => RegD
        COND   I+  -BWL RegN  RegD offs  offs offs | LDR/STR
        COND    +  - WL RegN  RegD        xx  RegM | LDRH/STRH/LDRSB/LDRSH depending on Op(xx)
        COND    +  - WL RegN  RegD imm_   xx  imm_ | LDRH/STRH/LDRSB/LDRSH depending on Op(xx) with Offset=imm_imm_
        COND    +  -RWL RegN  regs regs  regs regs | LDM/STM with register list regsregsregsregs
        COND        B   RegN  RegD            RegM | SWP
        COND       imm_ imm_  imm_ imm_  imm_ imm_ | SWI with comment
        COND       CPOP RegN  RegD CPID  xxx  RegM | CDP with CoCPU Op4(CPOP) and CP Info xxx
        COND       yyyL CprN  RegD CPID  xxx  CprM | MRC/MCR with CoCPU Op3(yyy) and CP Info xxx
        COND 110+  -NWL RegN  CprD CPID  imm_ imm_ | LDC/STC with unsigned Immediate
        COND    ?  ???? ????  ???? ????  ???  ???? | Unknown Instruction
        
    Full Instructions:
        .... ....  .... ....  .... ....  .... ....
        COND 0001  0010 1111  1111 1111  0001 RegM | BX #map: RegN => RegM
        COND 101F  imm_ imm_  imm_ imm_  imm_ imm_ | B/BL with signed offset
        COND 00Ix  xxxS RegN  RegD shft  shft shft | Data Processing Op(xxxx)
        COND 0001  0P00 1111  RegD 0000  0000 0000 | MRS
        COND 0001  0P10 1001  1111 0000  0000 RegM | MSR by RegM
        COND 00I1  0P10 1000  1111 shft  shft shft | MSR by imm
        COND 0000  00AS RegN  RegD RegS  1001 RegM | MUL/MLA #map: RegN <=> RegD
        COND 0000  1UAS RegN  RegD RegS  1001 RegM | MULL/MLAL #map: RdHi => RegN, RdLo => RegD
        COND 01I+  -BWL RegN  RegD offs  offs offs | LDR/STR
        COND 000+  -0WL RegN  RegD 0000  1xx1 RegM | LDRH/STRH/LDRSB/LDRSH depending on Op(xx)
        COND 000+  -1WL RegN  RegD imm_  1xx1 imm_ | LDRH/STRH/LDRSB/LDRSH depending on Op(xx) with Offset=imm_imm_
        COND 100+  -RWL RegN  regs regs  regs regs | LDM/STM with register list regsregsregsregs
        COND 0001  0B00 RegN  RegD 0000  1001 RegM | SWP
        COND 1111  imm_ imm_  imm_ imm_  imm_ imm_ | SWI with comment
        COND 1110  CPOP CprN  CprD CPID  xxx0 CprM | CDP with CoCPU Op4 CPOP and CP Info xxx
        COND 1110  yyyL CprN  RegD CPID  xxx1 CprM | MRC/MCR with CoCPU Op3 yyy and CP Info xxx
        COND 110+  -NWL RegN  CprD CPID  imm_ imm_ | LDC/STC with unsigned Immediate
        COND 011?  ???? ????  ???? ????  ???1 ???? | Unknown Instruction
    
    Bit Flags:
        I: 1=shftIsRegister,  0=shftIsImmediate
        F: 1=BranchWithLink,  0=BranchWithoutLink
        +: 1=PreIndexing,     0=PostIndexing
        -: 1=AddOffset,       0=SubtractOffset
        P: 1=SPSR,            0=CPSR
        U: 1=Signed,          0=Unsigned
        B: 1=TransferByte,    0=TransferWord
        R: 1=ForceUserMode,   0=NoForceUserMode
        N: 1=TransferAllRegs, 0=TransferSingleReg
        A: 1=Accumulate,      0=DoNotAccumulate
        W: 1=AutoIncrement,   0=NoWriteBack
        S: 1=SetFlags,        0=DoNotSetFlags
        L: 1=Load,            0=Store
        
    Shift format:
        I=0: shft shft shft
             _imm _op0 RegM // RegM = RegM SHIFT(op) _imm_
             RegS 0op1 RegM // RegM = RegM SHIFT(op) RegS
             
        I=1: shft shft shft
             xxxx imm_ imm_ // Immediate = imm_imm_ ROR 2*xxxx
    
    Offset format:
        I=0: offs offs offs
             imm_ imm_ imm_ // Immediate unsigned offset.
        
        I=1: offs offs offs
             _imm _op0 RegM // RegM = RegM SHIFT(op) _imm_
             RegS 0op1 RegM // RegM = RegM SHIFT(op) RegS
*/
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
    is_data_processing: bool,
    has_immediate: bool,
    shift_from_reg: bool,
    signed: bool,
    set_flags: bool,
    pre_indexed: bool,
    auto_increment: bool,
    force_usermode: bool,
    sub_offset_reg: bool,
    spsr: bool,
}

impl ArmInstruction {
    /// Creates an invalid instruction.
    pub fn new() -> ArmInstruction {
        ArmInstruction {
            immediate:          0,
            cond:               ArmCondition::AL,
            opcode:             ArmOpcode::Invalid,
            shift_op:           ArmBarrelShifterOp::LSL(0),
            Rd:                 0,
            Rn:                 0,
            Rs:                 0,
            Rm:                 0,
            is_data_processing: false,
            has_immediate:      false,
            shift_from_reg:     false,
            signed:             false,
            set_flags:          false,
            pre_indexed:        false,
            auto_increment:     false,
            force_usermode:     false,
            sub_offset_reg:     false,
            spsr:               false,
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
            // Possible bug, as MLA/MUL instructions
            // with a set sign bit will be accepted here.
            tmp.decode_multiplication(raw);
        }
        // Single data transfer?
        else if (raw & 0x0C000000_u32) == 0x04000000_u32 {
            tmp.decode_single_data_transfer(raw);
        }
        // Single Swap?
        else if (raw & 0x0FB00FF0_u32) == 0x01000090_u32 {
            tmp.Rn = ((raw >> 16) & 0b1111) as u8;
            tmp.Rd = ((raw >> 12) & 0b1111) as u8;
            tmp.Rm = ((raw >>  0) & 0b1111) as u8;
            tmp.opcode = if (raw & (1 << 22)) != 0 { ArmOpcode::SWPB } else { ArmOpcode::SWPW };
        }
        // Halfword or signed data transfer?
        else if (raw & 0x0E000090_u32) == 0x00000090_u32 {
            tmp.decode_halfword_and_signed_data_transfer(raw);
        }
        // Block data transfer?
        else if (raw & 0x0E000000_u32) == 0x08000000_u32 {
            tmp.decode_block_data_transfer(raw);
        }
        // Software Interrupt?
        else if (raw & 0x0F000000_u32) == 0x0F000000_u32 {
            // Decode comment as well, in case I want to use it for... stuff?
            tmp.immediate     = (raw & 0x00FFFFFF_u32) as i32;
            tmp.has_immediate = true;
            tmp.opcode        = ArmOpcode::SWI;
        }
        // Co-Processor data operations?
        else if (raw & 0x0F000010_u32) == 0x0E000000_u32 {
            tmp.decode_cp_data_op(raw);
        }
        // TODO co-processor and undefined
        // Undefined or unimplemented opcode.
        else {
            tmp.immediate = raw as i32;
            tmp.opcode    = ArmOpcode::Invalid;
            error!("Decoding invalid or unimplemented instruction: {:#8X}", raw);
        }
        
        // Done decoding.
        tmp
    }
    
    
    #[allow(non_snake_case)]
    fn decode_Rm_shifting(&mut self, raw: u32) {
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
    
    fn decode_data_processing(&mut self, raw: u32) {
        self.is_data_processing = true;
        
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
            self.decode_Rm_shifting(raw);
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
    
    fn decode_single_data_transfer(&mut self, raw: u32) {
        // Decode flags.
        self.has_immediate  = (raw & (1 << 25)) == 0;
        self.pre_indexed    = (raw & (1 << 24)) != 0;
        let up              = (raw & (1 << 23)) != 0;
        let byte            = (raw & (1 << 22)) != 0;
        self.auto_increment = (raw & (1 << 21)) != 0;
        let load            = (raw & (1 << 20)) != 0;
        
        // Decode operands.
        self.Rn = ((raw >> 16) & 0b1111) as u8;
        self.Rd = ((raw >> 12) & 0b1111) as u8;
        
        // Decode offset.
        if self.has_immediate {
            let x = (raw & 0x0FFF) as i32;
            self.immediate = if up { x } else { -x };
        } else {
            self.decode_Rm_shifting(raw);
        }
        
        // And decode opcode.
        self.opcode = match (byte, load) {
            (false, false) => ArmOpcode::STRW,
            (false, true ) => ArmOpcode::LDRW,
            (true,  false) => ArmOpcode::STRB, // Unsigned load.
            (true,  true ) => ArmOpcode::LDRB, // Unsigned load.
        };
    }
    
    fn decode_halfword_and_signed_data_transfer(&mut self, raw: u32) {
        // Decode flags.
        self.pre_indexed    = (raw & (1 << 24)) != 0;
        let up              = (raw & (1 << 23)) != 0;
        let offset          = (raw & (1 << 22)) != 0;
        self.auto_increment = (raw & (1 << 21)) != 0;
        let load            = (raw & (1 << 20)) != 0;
        self.signed         = (raw & (1 <<  6)) != 0;
        
        // Decode operands.
        self.Rn = ((raw >> 16) & 0b1111) as u8;
        self.Rd = ((raw >> 12) & 0b1111) as u8;
        let ohi = ((raw >>  4) & 0x00F0) as u8;
        let olo = ((raw >>  0) & 0b1111) as u8;
        
        // Load offset (register).
        if (!offset) & (ohi != 0) { warn!("Non-zero offset in non-offset halfword data transfer."); }
        if offset {
            self.immediate = (ohi | olo) as i32;
            if !up { self.immediate = -self.immediate; }
            self.has_immediate = true;
        } else {
            self.Rm = olo;
            self.sub_offset_reg = !up;
        }
        
        // Decode opcode.
        self.opcode = match (raw >> 5) & 0b11 {
            0     => unimplemented!(), // TODO SWP instruction? Wtf?
            2     => if load { ArmOpcode::LDRB } else { ArmOpcode::STRB },
            1 | 3 => if load { ArmOpcode::LDRH } else { ArmOpcode::STRH },
            _ => unreachable!(),
        };
    }
    
    fn decode_block_data_transfer(&mut self, raw: u32) {
        // Decode flags.
        self.pre_indexed    = (raw & (1 << 24)) != 0;
        self.sub_offset_reg = (raw & (1 << 23)) == 0; // Decrement addressing.
        self.force_usermode = (raw & (1 << 22)) != 0;
        self.auto_increment = (raw & (1 << 21)) != 0;
        let load            = (raw & (1 << 20)) != 0;
        
        // Decode operand and register list.
        self.Rn        = ((raw >> 16) & 0b1111) as u8;
        self.immediate = ((raw >>  0) & 0xFFFF) as i32;
        self.has_immediate = true;
        
        // Decode opcode.
        self.opcode = if load { ArmOpcode::LDM } else { ArmOpcode::STM };
    }
    
    fn decode_cp_data_op(&mut self, raw: u32) {
        // Decode registers.
        self.Rn = ((raw >> 16) & 0b1111) as u8;
        self.Rd = ((raw >> 12) & 0b1111) as u8;
        self.Rm = ((raw >>  0) & 0b1111) as u8;
        
        // Decode information and CP#.
        self.has_immediate = true;
        self.immediate     = ((raw >> 8) & 0b1111) as u8; // Co-Processor number.
        self.Rs            = ((raw >> 5) & 0b0111) as u8; // Co-Processor information.
        
        // Decode CP opcode.
        self.opcode = ArmOpcode::CDP(((raw >> 20) & 0b1111) as u8);
    }
}
