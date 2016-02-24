

use std::io::Write;
use std::fs::File;
use std::path::Path;
use std::sync::Mutex;
use std::cell::RefCell;
use std::thread;
use log::{set_logger, Log, LogMetadata, LogRecord, LogLevel, LogLevelFilter, SetLoggerError};


pub struct ConsoleFileLogger {
    pub file: Option<Mutex<RefCell<File>>>,
    pub verbose: bool,
}

impl Log for ConsoleFileLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        let min_level = if self.verbose { LogLevel::Info } else { LogLevel::Trace };
        metadata.level() <= min_level
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            let loc = record.location();
            let msg = format!(
                "[TID={tid}]\t{level} [{file}:{line} - {module}]\n\t\t-- {message}\n",
                tid     = thread::current().name().unwrap_or("<?>"),
                level   = record.level(),
                file    = loc.file(),
                line    = loc.line(),
                module  = loc.module_path(),
                message = format!("{}", record.args()).replace("\n","\n\t\t   ")
            );
            
            if let Some(f) = self.file.as_ref() {
                let tmp = f.lock().unwrap();
                writeln!(*(tmp.borrow_mut()), "{}", msg).unwrap();
            }
            println!("{}", msg);
        }
    }
}

pub fn init_with(file: &Path, verbose: bool) -> Result<(), SetLoggerError> {
    set_logger(|max_log_level| {
        max_log_level.set(LogLevelFilter::Trace);
        box ConsoleFileLogger {
            file: Some(Mutex::new(RefCell::new(File::create(file).unwrap()))),
            verbose: verbose
        }
    })
}
