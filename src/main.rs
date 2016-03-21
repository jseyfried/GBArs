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
extern crate term;

#[cfg(test)]
extern crate test;

use argparse::{ArgumentParser, Print, Parse, ParseOption, StoreTrue, StoreFalse, StoreOption};
use std::path::PathBuf;
use std::process;

pub mod logger;
pub mod hardware;


/// Set of values configurable by the command line.
///
/// Execute `GBArs -h` or `GBArs --help` to print
/// a list of all supported command line arguments.
pub struct CmdLineArgs {
    /// Accepts `--bios FILE`
    ///
    /// The ROM file will be loaded immediately after
    /// initialising the emulator.
    pub bios_file_path: Option<PathBuf>,

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

    /// Accepts `--dasm-thumb INST`.
    ///
    /// Disassembles a single THUMB instruction
    /// and logs the result.
    pub single_disasm_thumb: Option<String>,

    /// Accepts `-v` or `--verbose` as `true`.
    ///
    /// Also accepts `-q` or `--quiet` as `false`, which is the default value.
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

    /// Accepts `-x` or `--exit` as `true`.
    ///
    /// If `true`, exits after handling the command line
    /// parameters instead of entering the main program.
    pub exit: bool,

    /// Accepts `-S` or `--optimise-swi` as `true`.
    ///
    /// Also accepts `-s` or `--emulate-swi` as `false`, which is the default value.
    ///
    /// If `true`, any execution of a `SWI` instruction
    /// will first check for an optimised implementation
    /// of BIOS functions provided by the emulator. If
    /// such an implementation exists, this one will be
    /// executed instead of emulating the actual BIOS
    /// code.
    pub optimise_swi: bool,
}

impl Default for CmdLineArgs {
    fn default() -> CmdLineArgs {
        CmdLineArgs {
            bios_file_path: None,
            rom_file_path: None,
            log_file_path: PathBuf::from("./GBArs.log"),
            single_disasm_arm: None,
            single_disasm_thumb: None,
            verbose: cfg!(debug_assertions), // Default to `true` while testing.
            colour: true,
            exit: false,
            optimise_swi: false,
        }
    }
}


fn main() {
    // Build command line parser.
    let mut args = CmdLineArgs::default();
    parse_command_line(&mut args);
    configure_logging(&args);

    // Prepare the GBA and handle oneshot commands.
    let mut gba = hardware::Gba::new();
    configure_gba_from_command_line(&mut gba, &args);
    handle_oneshot_commands(&args, &gba);

    // Exit early?
    if args.exit { trace!("Exiting early."); process::exit(0); }
}


fn parse_command_line(args: &mut CmdLineArgs) {
    let mut parser = ArgumentParser::new();
    parser.set_description("A GBA emulator written in Rust.");
    parser.add_option(&["-V", "--version"],
                      Print(format!("GBArs v{}", env!("CARGO_PKG_VERSION"))),
                      "Show current version.");
    parser.refer(&mut args.bios_file_path)
          .add_option(&["--bios"], ParseOption, "Path to a BIOS file to load.")
          .metavar("PATH");
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
    parser.refer(&mut args.single_disasm_thumb)
          .add_option(&["--dasm-thumb"], StoreOption,
                      "Prints the disassembly of the given THUMB state instruction. \
                       The instruction must be a hex number without base, e.g. 01F7, \
                       in Big Endian format, i.e. the most significant byte is left.")
          .metavar("INST");
    parser.refer(&mut args.verbose)
          .add_option(&["-v","--verbose"], StoreTrue, "Log extra messages and information.")
          .add_option(&["-q","--quiet"], StoreFalse, "Log with less messages and information. (default)");
    parser.refer(&mut args.colour)
          .add_option(&["-c","--with-colour"], StoreTrue, "Enable terminal logging with ANSI colour codes. (default)")
          .add_option(&["-k","--without-colour"], StoreFalse, "Disable terminal logging with ANSI colour codes.");
    parser.refer(&mut args.exit)
          .add_option(&["-x","--exit"], StoreTrue, "Exit early after handling the command line arguments.");
    parser.refer(&mut args.optimise_swi)
          .add_option(&["-S","--optimise-swi"], StoreTrue, "Enable optimised BIOS functions.")
          .add_option(&["-s","--emulate-swi"], StoreFalse, "Disable optimised BIOS functions. (default)");
    parser.parse_args_or_exit();
}


fn configure_logging(args: &CmdLineArgs) {
    let p = args.log_file_path.as_path();
    logger::init_with(&p, args.verbose, args.colour).unwrap();
    info!("Logging to file `{}`.", p.display());
}


fn handle_oneshot_commands(args: &CmdLineArgs, gba: &hardware::Gba) {
    // Single instructions to disassemble?
    if let Some(ref x) = args.single_disasm_arm   { disasm_arm(x); }
    if let Some(ref x) = args.single_disasm_thumb { disasm_thumb(x); }
}

fn disasm_arm(x: &String) {
    match u32::from_str_radix(x.as_str(), 16) {
        Ok(i) => { match hardware::cpu::ArmInstruction::decode(i) {
            Ok(inst) => info!("DASM ARM:\t{}", inst),
            Err(e)   => info!("DASM ARM invalid - {}", e),
        };},
        Err(e) => { error!("DASM ARM: {}\nRun `GBArs --help` for details.", e); },
    }
}

fn disasm_thumb(x: &String) {
    error!("DASM THUMB: Not yet implemented!");
    // TODO implement THUMB state instructions.
}


fn configure_gba_from_command_line(gba: &mut hardware::Gba, args: &CmdLineArgs) {
    // If a BIOS file is given, load it into the BIOS ROM area.
    if let Some(ref fp) = args.bios_file_path {
        if let Err(e) = gba.bios_mut().load_from_file(fp.as_path()) {
            error!("Failed loading the BIOS file:\n{}", e);
        } else {
            info!("Loaded the BIOS ROM from file.");
        }
    }

    // Load ROM now if a path is given.
    if let Some(ref fp) = args.rom_file_path {
        let res = gba.game_pak_mut().load_rom_from_file(fp.as_path());
        if let Err(e) = res {
            error!("Failed loading the ROM file:\n{}", e);
        } else {
            let gpak  = gba.game_pak();
            let gpakh = (*gpak).header();
            info!("Loaded the game {}.", gpakh);
            debug!("Header valid? {}", gpakh.complement_check());
        }
    }

    // Configure the CPU.
    gba.cpu_arm7tdmi_mut().set_swi_optimised(args.optimise_swi);
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
