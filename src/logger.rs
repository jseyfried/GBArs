// License below.
//! GBArs' logging implementation.
#![cfg_attr(feature="clippy", warn(result_unwrap_used, option_unwrap_used, print_stdout))]
#![cfg_attr(feature="clippy", warn(single_match_else, string_add, string_add_assign))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]
#![warn(missing_docs)]

use std::io::Write;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::sync::Mutex;
use std::cell::RefCell;
use std::thread;
use log::{set_logger, Log, LogMetadata, LogRecord, LogLevel, LogLevelFilter, SetLoggerError};

/// A logger that forwards log messages to `stdout` and a log file.
pub struct ConsoleFileLogger {
    file: Option<Mutex<RefCell<File>>>,
    verbose: bool,
    colour: bool,
}

impl Log for ConsoleFileLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        let min_level = if self.verbose { LogLevel::Trace } else { LogLevel::Info };
        metadata.level() <= min_level
    }

    #[cfg_attr(feature="clippy", allow(print_stdout))]
    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            // Prepare some common message sections in case of colouring.
            let cur = thread::current();
            let tid = cur.name().unwrap_or("<?>");
            let loc = record.location();
            let loc = format!("[{}:{} - {}]", loc.file(), loc.line(), loc.module_path());
            let fmt = format!("{}", record.args()).replace("\n","\n\t\t   ");

            // Build a common log message for both targets.
            let mut msg = format!("[TID={}]\t{}\t{}\n\t\t-- {}\n", tid, record.level(), loc, fmt);

            // Log to file.
            if let Some(f) = self.file.as_ref() {
                let tmp = f.lock().unwrap();
                writeln!(*(tmp.borrow_mut()), "{}", msg).unwrap();
            }
            else { msg.push_str("\n\x1B[31m\x1B[1mNo log file!\x1B[0m"); }

            // Log to stdout.
            if !self.colour { println!("{}", msg); }
            else {
                // Colourising is only done for terminals.
                println!(
                    "\x1B[0m\x1B[2m[TID={}]\t{}{}\x1B[0m\x1B[2m\t{}\x1B[1m\n\t\t-- {}\x1B[0m\n",
                    tid, match record.level() {
                        LogLevel::Error => "\x1B[31m\x1B[1m", // Bold, red.
                        LogLevel::Warn  => "\x1B[33m\x1B[1m", // Bold, yellow.
                        LogLevel::Info  => "\x1B[32m\x1B[1m", // Bold, green.
                        _               => "\x1B[34m\x1B[1m", // Bold, blue.
                    }, record.level(), loc, fmt
                );
            }
        }
    }
}


/// Initialises the logging library to use a `ConsoleFileLogger`.
///
/// # Params
/// - `file`: Path to the log file to write to.
/// - `verbose`: If `false`, ignores debug and trace messages.
/// - `colour`: If `true`, colourises the `stdout` output using escape codes.
///
/// # Returns
/// - `Ok` if the logger has been created successfully.
/// - `Err` otherwise.
pub fn init_with(file: &Path, verbose: bool, colour: bool) -> Result<(), SetLoggerError> {
    set_logger(|max_log_level| {
        max_log_level.set(LogLevelFilter::Trace);
        box ConsoleFileLogger {
            file: match OpenOptions::new().write(true).truncate(true).open(file) {
                Ok(f)  => Some(Mutex::new(RefCell::new(f))),
                Err(_) => None,
            },
            verbose: verbose,
            colour: colour,
        }
    })
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
