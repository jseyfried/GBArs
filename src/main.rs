// License below.
//! # GBArs
//!
//! A GameBoy Advance emulator written in Rust.
//!
//! Thanks to a guy named Ferris and his project [Rustendo 64][1],
//! many people got motivated to write their own emulators in Rust. Even I wasn't
//! spared, so here it is, my GBA emulator.
//!
//! And why GBA?
//!
//! - It is ARM-based and ARM is sexy.
//! - I want to play Metroid Zero Mission and Fusion with it.
//! - It can handle GBC games as well.
//!
//! [1]: https://github.com/yupferris/rustendo64
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![feature(box_syntax, associated_consts)]
#![warn(missing_docs)]

#[macro_use]
extern crate log;
extern crate argparse;
extern crate byteorder;

use argparse::{ArgumentParser, Print, Parse, ParseOption, StoreTrue, StoreFalse, StoreOption};
use std::path::PathBuf;

pub mod logger;
pub mod hardware;


/// Set of values configurable by the command line.
///
/// Execute `GBArs -h` or `GBArs --help` to print
/// a list of all supported command line arguments.
pub struct CmdLineArgs {
    /// Accepts `--rom FILE`.
    ///
    /// The ROM file will be loaded immediately after
    /// initialising the emulator.
    pub rom_file_path: Option<PathBuf>,
    
    /// Accepts `--log FILE`, defaults to `"./GBArs.log"`.
    pub log_file_path: PathBuf,
    
    /// Accepts `--dasm-arm INST`.
    ///
    /// Disassembles a single ARM instruction and
    /// logs the result.
    pub single_disasm_arm: Option<String>,
    
    /// Accepts `-v` or `--verbose` as `true`.
    ///
    /// If `false`, log messages of log level *debug*
    /// and *trace* will be ignored.
    pub verbose: bool,
    
    /// Accepts `-c` or `--with-colour` as `true`, which is the default value.
    ///
    /// Also accepts `-k` or `--without-colour` as `false`.
    ///
    /// If `true`, log messages sent to the console will be
    /// slightly colourised for improved readability.
    pub colour: bool,
}

impl Default for CmdLineArgs {
    fn default() -> CmdLineArgs {
        CmdLineArgs {
            rom_file_path: None,
            log_file_path: PathBuf::from("./GBArs.log"),
            single_disasm_arm: None,
            verbose: false,
            colour: true,
        }
    }
}


fn main() {
    // Build command line parser.
    let mut args = CmdLineArgs::default();
    parse_command_line(&mut args);
    configure_logging(&args);
    handle_oneshot_commands(&args);
    
    // Prepare the GBA.
    let mut gba = hardware::Gba::new();
    configure_gba_from_command_line(&mut gba, &args);
}


fn parse_command_line(args: &mut CmdLineArgs) {
    let mut parser = ArgumentParser::new();
    parser.set_description("A GBA emulator written in Rust.");
    parser.add_option(&["-V", "--version"],
                      Print(format!("GBArs v{}", env!("CARGO_PKG_VERSION"))),
                      "Show current version.");
    parser.refer(&mut args.rom_file_path)
          .add_option(&["--rom"], ParseOption, "Path to a ROM file to load.")
          .metavar("PATH");
    parser.refer(&mut args.log_file_path)
          .add_option(&["--log"], Parse, "Custom path for the log file.")
          .metavar("PATH");
    parser.refer(&mut args.single_disasm_arm)
          .add_option(&["--dasm-arm"], StoreOption,
                      "Prints the disassembly of the given ARM state instruction. \
                       The instruction must be a hex number without base, e.g. 01F7344, \
                       in Big Endian format, i.e. the most significant byte is left.")
          .metavar("INST");
    parser.refer(&mut args.verbose)
          .add_option(&["-v","--verbose"], StoreTrue, "Log extra messages and information.");
    parser.refer(&mut args.colour)
          .add_option(&["-c","--with-colour"], StoreTrue, "Enable terminal logging with colour codes. (default)")
          .add_option(&["-k","--without-colour"], StoreFalse, "Disable terminal logging with colour codes.");
    parser.parse_args_or_exit();
}


fn configure_logging(args: &CmdLineArgs) {
    let p = args.log_file_path.as_path();
    logger::init_with(&p, args.verbose, args.colour).unwrap();
    info!("Logging to file `{}`.", p.display());
}


fn handle_oneshot_commands(args: &CmdLineArgs) {
    // Single ARM instruction to disassemble?
    if let Some(ref x) = args.single_disasm_arm {
        match u32::from_str_radix(x.as_str(), 16) {
            Ok(i) => { match hardware::cpu::ArmInstruction::decode(i as i32) {
                Ok(inst) => info!("DASM ARM:\t{}", inst),
                Err(e)   => info!("DASM ARM invalid - {}", e),
            };},
            Err(e) => { error!("{}", e); },
        };
    }
}


fn configure_gba_from_command_line(gba: &mut hardware::Gba, args: &CmdLineArgs) {
    // Load ROM now if a path is given.
    if let Some(ref fp) = args.rom_file_path {
        if let Err(e) = gba.load_rom_from_file(fp.as_path()) {
            error!("Failed loading the ROM file:\n{}", e);
            return;
        }
        info!("Loaded the game {}.", gba.rom_header());
        debug!("Header valid? {}", gba.rom_header().complement_check());
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
