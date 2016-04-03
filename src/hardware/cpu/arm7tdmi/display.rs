// License below.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use super::*;
use std::fmt;

impl fmt::Display for Arm7Tdmi {
    /// Shows the current CPU state with all its registers and what not.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Show CPSR and SPSR.
        try!(write!(f, "Arm7Tdmi\n\n- Register Set\n\tCPSR {}\tSPSR ", self.cpsr));
        if self.mode == Mode::User { try!(write!(f, "[none]\n")); }
        else { try!(write!(f, "{}\n", self.spsr[self.mode as u8 as usize])); }

        // Show all 16 GPRs in a nice table.
        for i in 0..16 {
            if (i % 4) == 0 { try!(write!(f, "\n\t")); }
            try!(write!(f, "{}[{:08X}]\t", Arm7Tdmi::DEBUG_REGISTER_NAMES[i], self.gpr[i]));
        }

        // Show the current pipeline state.
        try!(write!(f, "\n\n- Pipeline State\n\t\
                                ARM   Fetch:  {:#010X}\n\t\
                                ARM   Decode: {}\n\t\n\t\
                                THUMB Fetch:      {:#06X}\n\t\
                                THUMB Decode:     {}\n\t",
            self.fetched_arm, self.decoded_arm, self.fetched_thumb, self.decoded_thumb
        ));

        // Show extra settings 'n' stuff.
        write!(f, "\n- CPU Settings\n\tCurrent Delay:\t{}\n\tOptimise SWI:\t{}",
            self.delay_cycles, self.optimise_swi
        )
    }
}

impl Arm7Tdmi {
    const DEBUG_REGISTER_NAMES: &'static [&'static str] = &[
        "R0:  ", "R1:  ", "R2:  ", "R3:  ", "R4:  ", "R5:  ", "R6:  ", "R7:  ",
        "R8:  ", "R9:  ", "R10: ", "R11: ", "R12: ", "SP:  ", "LR:  ", "PC:  "
    ];

    const REGISTER_NAMES: &'static [&'static str] = &[
        "R0", "R1", "R2", "R3", "R4", "R5", "R6", "R7",
        "R8", "R9", "R10", "R11", "R12", "SP", "LR", "PC"
    ];

    /// Get the name corresponding to a given register index.
    pub fn register_name(i: usize) -> &'static str {
        debug_assert!(i < Arm7Tdmi::REGISTER_NAMES.len());
        Arm7Tdmi::REGISTER_NAMES[i]
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
