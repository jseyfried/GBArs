// License below.
//! Implements a diff viewer for the ARM7TDMI's register state.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use super::hardware;
use super::term;
use std::io;
use std::io::Write;
use std::str::SplitWhitespace;

/// Implements a debug REPL for the GBA emulator.
///
/// REPL stands for **R**ead, **E**val, **P**rint, **L**oop,
/// so all it does is running the emulator step by step waiting
/// for user inputs.
pub struct GbaRepl {
    colour: bool,
}

impl GbaRepl {
    /// Creates a new REPL without running it.
    pub fn new() -> GbaRepl {
        GbaRepl {
            colour: true,
        }
    }

    /// Configure whether terminal outputs should be colourised.
    pub fn with_colour(&mut self, c: bool) -> &mut GbaRepl { self.colour = c; self }

    /// Runs the REPL until the user quits, an error occurred,
    /// or until the emulated program ends.
    pub fn run(&mut self, gba: &mut hardware::Gba) -> Result<(), hardware::GbaError> {
        // Prepare everything we need.
        let mut terminal = term::stdout().expect("Failed grabbing a terminal handle!");
        let mut diff     = hardware::cpu::Arm7TdmiDiff::new();
        gba.cpu_arm7tdmi_mut().reset();
        diff.diff(gba.cpu_arm7tdmi());
        diff.print(&mut terminal).unwrap_or(());
        let mut input = String::new();

        // Now run the actual REPL.
        loop {
            let mut s = match self.input_prompt(&mut terminal, &mut input) {
                Ok(s) => s, Err(e) => { error!("{}", e); break }, // Abort loop on error.
            };

            match s.next() {
                Some("q") => break,
                _ => write!(terminal, "\t\t<What?>").unwrap_or_else(|e| { error!("{}", e); }),
            }
        }
        Ok(())
    }

    fn input_prompt<'a>(&'a self, terminal: &mut Box<term::StdoutTerminal>, input: &'a mut String)
    -> io::Result<SplitWhitespace> {
        try!(write!(terminal, "\t"));
        if self.colour {
            terminal.fg(term::color::BLACK).unwrap_or(());
            terminal.bg(term::color::WHITE).unwrap_or(());
        }
        try!(write!(terminal, "[? = Help, q = Quit, hex A..B, run N]"));
        terminal.reset().unwrap_or(());
        try!(write!(terminal, "\n\t> "));
        io::stdout().flush().unwrap();

        input.clear();
        try!(io::stdin().read_line(input));
        try!(write!(terminal, "\n"));
        Ok(input.trim().split_whitespace())
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
