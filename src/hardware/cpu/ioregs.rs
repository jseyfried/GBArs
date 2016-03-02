// License below.
//! Implements emulation utilities for the GBA's memory-mapped IO registers.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

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
