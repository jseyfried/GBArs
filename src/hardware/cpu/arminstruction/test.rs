// License below.
#![allow(missing_docs)]

use std::fmt::Write;
use super::ArmInstruction;

const INSTRUCTIONS_RAW: &'static [i32] = &[
    // Use SWI to check condition decoding.
    0b0000_1111_011101110111011101110111_u32 as i32,
    0b0001_1111_011101110111011101110111_u32 as i32,
    0b0010_1111_011101110111011101110111_u32 as i32,
    0b0011_1111_011101110111011101110111_u32 as i32,
    0b0100_1111_011101110111011101110111_u32 as i32,
    0b0101_1111_011101110111011101110111_u32 as i32,
    0b0110_1111_011101110111011101110111_u32 as i32,
    0b0111_1111_011101110111011101110111_u32 as i32,
    0b1000_1111_011101110111011101110111_u32 as i32,
    0b1001_1111_011101110111011101110111_u32 as i32,
    0b1010_1111_011101110111011101110111_u32 as i32,
    0b1011_1111_011101110111011101110111_u32 as i32,
    0b1100_1111_011101110111011101110111_u32 as i32,
    0b1101_1111_011101110111011101110111_u32 as i32,
    0b1110_1111_011101110111011101110111_u32 as i32,
    0b1111_1111_011101110111011101110111_u32 as i32,

    // Test BX, B, BL.
    0b0000_000100101111111111110001_0111_u32 as i32,
    0b0000_101_0_111111111111111111111101_u32 as i32,
    0b0000_101_0_000000000000000000000001_u32 as i32,
    0b0000_101_1_111111111111111111111101_u32 as i32,
    0b0000_101_1_000000000000000000000001_u32 as i32,

    // Test Unknown.
    0b0000_011_01100110011001100110_1_0110_u32 as i32,

    // Data Processing.
    0b0000_00_1_0000_0_0001_0010_0011_01000101_u32 as i32,
    0b0000_00_0_0001_1_0001_0010_00111_00_0_0011_u32 as i32,
    0b0000_00_0_0010_0_0001_0010_00111_01_0_0011_u32 as i32,
    0b0000_00_0_0011_0_0001_0010_00111_10_0_0011_u32 as i32,
    0b0000_00_0_0100_0_0001_0010_00111_11_0_0011_u32 as i32,
    0b0000_00_0_0101_0_0001_0010_00000_11_0_0011_u32 as i32,
    0b0000_00_0_0110_0_0001_0010_00000_00_0_0011_u32 as i32,
    0b0000_00_0_0111_0_0001_0010_0100_0_00_1_0011_u32 as i32,
    0b0000_00_0_1000_0_0001_0010_0100_0_01_1_0011_u32 as i32,
    0b0000_00_0_1001_0_0001_0010_0100_0_10_1_0011_u32 as i32,
    0b0000_00_0_1010_0_0001_0010_0100_0_11_1_0011_u32 as i32,
    0b0000_00_0_1011_0_0001_0010_00000_00_0_0011_u32 as i32,
    0b0000_00_0_1100_0_0001_0010_00000_00_0_0011_u32 as i32,
    0b0000_00_0_1101_0_0001_0010_00000_00_0_0011_u32 as i32,
    0b0000_00_0_1110_0_0001_0010_00000_00_0_0011_u32 as i32,
    0b0000_00_0_1111_0_0001_0010_00000_00_0_0011_u32 as i32,

    // MRS and MSR.
    0b0000_00010_0_001111_0001_000000000000_u32 as i32,
    0b0000_00010_1_001111_0001_000000000000_u32 as i32,
    0b0000_00010_0_101001111100000000_0010_u32 as i32,
    0b0000_00_0_10_0_1010001111_00000000_0111_u32 as i32,
    0b0000_00_0_10_1_1010001111_11111111_0111_u32 as i32,
    0b0000_00_1_10_0_1010001111_0010_00001111_u32 as i32,

    // Test MUL and MLA.
    0b0000_000000_0_0_0001_0010_0011_1001_0100_u32 as i32,
    0b0000_000000_0_1_0001_0010_0011_1001_0100_u32 as i32,
    0b0000_000000_1_0_0001_0010_0011_1001_0100_u32 as i32,
    0b0000_000000_1_1_0001_0010_0011_1001_0100_u32 as i32,

    // Test MULL and MLAL.
    0b0000_00001_0_0_0_0001_0010_0011_1001_0100_u32 as i32,
    0b0000_00001_0_0_1_0001_0010_0011_1001_0100_u32 as i32,
    0b0000_00001_0_1_0_0001_0010_0011_1001_0100_u32 as i32,
    0b0000_00001_0_1_1_0001_0010_0011_1001_0100_u32 as i32,
    0b0000_00001_1_0_0_0001_0010_0011_1001_0100_u32 as i32,
    0b0000_00001_1_0_1_0001_0010_0011_1001_0100_u32 as i32,
    0b0000_00001_1_1_0_0001_0010_0011_1001_0100_u32 as i32,
    0b0000_00001_1_1_1_0001_0010_0011_1001_0100_u32 as i32,

    // Test LDR and STR.
    0b0000_01_0_0_0_0_0_0_0001_0010_011101110111_u32 as i32,
    0b0000_01_0_0_0_0_0_1_0001_0010_011101110111_u32 as i32,
];

const EXPECTED_DISASSEMBLY: &'static str = "\
    0x0F777777\tswieq\t#0x777777\n\
    0x1F777777\tswine\t#0x777777\n\
    0x2F777777\tswihs\t#0x777777\n\
    0x3F777777\tswilo\t#0x777777\n\
    0x4F777777\tswimi\t#0x777777\n\
    0x5F777777\tswipl\t#0x777777\n\
    0x6F777777\tswivs\t#0x777777\n\
    0x7F777777\tswivc\t#0x777777\n\
    0x8F777777\tswihi\t#0x777777\n\
    0x9F777777\tswils\t#0x777777\n\
    0xAF777777\tswige\t#0x777777\n\
    0xBF777777\tswilt\t#0x777777\n\
    0xCF777777\tswigt\t#0x777777\n\
    0xDF777777\tswile\t#0x777777\n\
    0xEF777777\tswi\t#0x777777\n\
    0xFF777777\tswinv\t#0x777777\n\
    0x012FFF17\tbxeq\tR7\n\
    0x0AFFFFFD\tbeq\t#-4\n\
    0x0A000001\tbeq\t#12\n\
    0x0BFFFFFD\tbleq\t#-4\n\
    0x0B000001\tbleq\t#12\n\
    0x06CCCCD6\t<unknown>\n\
    0x02012345\tandeq\tR2, R1, #335544321\n\
    0x00312383\teoreqs\tR2, R1, R3, lsl #7\n\
    0x004123A3\tsubeq\tR2, R1, R3, lsr #7\n\
    0x006123C3\trsbeq\tR2, R1, R3, asr #7\n\
    0x008123E3\taddeq\tR2, R1, R3, ror #7\n\
    0x00A12063\tadceq\tR2, R1, R3, rrx\n\
    0x00C12003\tsbceq\tR2, R1, R3\n\
    0x00E12413\trsceq\tR2, R1, R3, lsl R4\n\
    0x01012433\ttsteq\tR1, R3, lsr R4\n\
    0x01212453\tteqeq\tR1, R3, asr R4\n\
    0x01412473\tcmpeq\tR1, R3, ror R4\n\
    0x01612003\tcmneq\tR1, R3\n\
    0x01812003\torreq\tR2, R1, R3\n\
    0x01A12003\tmoveq\tR2, R3\n\
    0x01C12003\tbiceq\tR2, R1, R3\n\
    0x01E12003\tmvneq\tR2, R3\n\
    0x010F1000\tmrseq\tR1, CPSR\n\
    0x014F1000\tmrseq\tR1, SPSR\n\
    0x0129F002\tmsreq\tCPSR, R2\n\
    0x0128F007\tmsreq\tCPSR_flg, R7\n\
    0x0168FFF7\tmsreq\tSPSR_flg, R7\n\
    0x0328F20F\tmsreq\tCPSR_flg, #0xF0000000\n\
    0x00012394\tmuleq\tR1, R4, R3\n\
    0x00112394\tmulseq\tR1, R4, R3\n\
    0x00212394\tmlaeq\tR1, R4, R3, R2\n\
    0x00312394\tmlaseq\tR1, R4, R3, R2\n\
    0x00812394\tumulleq\tR2, R1, R4, R3\n\
    0x00912394\tumullseq\tR2, R1, R4, R3\n\
    0x00A12394\tumlaleq\tR2, R1, R4, R3\n\
    0x00B12394\tumlalseq\tR2, R1, R4, R3\n\
    0x00C12394\tsmulleq\tR2, R1, R4, R3\n\
    0x00D12394\tsmullseq\tR2, R1, R4, R3\n\
    0x00E12394\tsmlaleq\tR2, R1, R4, R3\n\
    0x00F12394\tsmlalseq\tR2, R1, R4, R3\n\
    0x04012777\tstreq\tR2, [R1], #-1911\n\
    0x04112777\tldreq\tR2, [R1], #-1911\n\
";

#[test]
pub fn instruction_disassembly() {
    let mut dis = String::new();

    for inst in self::INSTRUCTIONS_RAW {
        writeln!(dis, "{}", ArmInstruction::decode(*inst).unwrap()).unwrap();
    }

    println!("\n========================\nExpected Disassembly:\n\n{}", self::EXPECTED_DISASSEMBLY);
    println!("\n========================\nGenerated Disassembly:\n\n{}", dis);

    assert!(dis == self::EXPECTED_DISASSEMBLY);
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
