

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![feature(box_syntax, associated_consts)]

#[macro_use]
extern crate log;
extern crate argparse;
extern crate byteorder;

use argparse::{ArgumentParser, Print, Parse, ParseOption, StoreTrue, StoreFalse, StoreOption};
use std::path::PathBuf;

mod logger;
mod hardware;


struct CmdLineArgs {
    rom_file_path: Option<PathBuf>,
    log_file_path: PathBuf,
    single_disasm_arm: Option<String>,
    verbose: bool,
    colour: bool,
}

impl Default for CmdLineArgs {
    //
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
