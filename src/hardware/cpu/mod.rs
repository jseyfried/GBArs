// License below.
//! Implements everything related to the GBA's CPUs.
//!
//! The GBA has two different CPUs installed:
//!
//! - **ARM7TDMI** capable of executing two instruction sets:
//!     - ARMv4 instruction set.
//!     - THUMB instruction set.
//! - **Zilog Z80** for GameBoy Color backwards compatiblity.
//!
//! Emulation utilities for these CPUs and their instruction
//! sets are implemented here.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

pub use self::arm7tdmi::*;
pub use self::arminstruction::*;
pub use self::armcondition::*;
pub use self::ioregs::*;

pub mod arm7tdmi;
pub mod arminstruction;
pub mod armcondition;
pub mod ioregs;


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
