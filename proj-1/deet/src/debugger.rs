use std::collections::HashMap;

use crate::debugger_command::DebuggerCommand;
use crate::inferior::Inferior;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use crate::inferior::Status;
use crate::dwarf_data::{DwarfData, Error as DwarfError};

fn parse_address(addr: &str) -> Option<usize> {
    let addr_without_0x = if addr.to_lowercase().starts_with("0x") {
        &addr[2..]
    } else {
        &addr
    };
    usize::from_str_radix(addr_without_0x, 16).ok()
}

pub struct Debugger {
    target: String,
    history_path: String,
    readline: Editor<()>,
    inferior: Option<Inferior>,
    debug_data: DwarfData,
    breakpoints_map: HashMap<usize, u8>, // addr: usize --- orig_byte: u8
}

impl Debugger {
    /// Initializes the debugger.
    pub fn new(target: &str) -> Debugger {
        // TODO (milestone 3): initialize the DwarfData
        let debug_data = match DwarfData::from_file(target) {
            Ok(val) => val,
            Err(DwarfError::ErrorOpeningFile) => {
                println!("Could not open file {}", target);
                std::process::exit(1);
            }
            Err(DwarfError::DwarfFormatError(err)) => {
                println!("Could not debugging symbols from {}: {:?}", target, err);
                std::process::exit(1);
            }
        };
        
        // for test breakpoints
        debug_data.print();

        let history_path = format!("{}/.deet_history", std::env::var("HOME").unwrap());
        let mut readline = Editor::<()>::new();
        // Attempt to load history from ~/.deet_history if it exists
        let _ = readline.load_history(&history_path);

        Debugger {
            target: target.to_string(),
            history_path,
            readline,
            inferior: None,
            debug_data: debug_data,
            breakpoints_map: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.get_next_command() {
                DebuggerCommand::Run(args) => {
                    // check that kill any existing inferiors before starting new ones
                    if self.inferior.is_some() {
                        self.inferior.as_mut().unwrap().kill();
                        self.inferior = None;
                    }
                    if let Some(inferior) = Inferior::new(&self.target, &args, &mut self.breakpoints_map) {
                        // Create the inferior
                        self.inferior = Some(inferior);
                        // TODO (milestone 1): make the inferior run
                        // You may use self.inferior.as_mut().unwrap() to get a mutable reference
                        // to the Inferior object
                        match self.inferior.as_mut().unwrap().continue_run(None, &mut self.breakpoints_map).unwrap() {
                            Status::Exited(exit_code) => {
                                println!("Child exited (status {})", exit_code);
                                self.inferior = None;
                            },
                            Status::Signaled(signal) => {
                                println!("Child exited due to signal {}", signal);
                                self.inferior = None;
                            },
                            Status::Stopped(signal, rip) => {
                                println!("Child stopped (signal {})", signal);
                                println!("Stopped at {}", self.debug_data.get_line_from_addr(rip).unwrap());
                            },
                        }
                    } else {
                        println!("Error starting subprocess");
                    }
                }
                DebuggerCommand::Quit => {
                    if self.inferior.is_some() {
                        self.inferior.as_mut().unwrap().kill();
                        self.inferior = None;
                    }
                    
                    return;
                }
                DebuggerCommand::Continue => {
                    if self.inferior.is_none() {
                        println!("Error continuing precess when no inferior is running");
                    } else {
                        match self.inferior.as_mut().unwrap().continue_run(None, &mut self.breakpoints_map).unwrap() {
                            Status::Exited(exit_code) => {
                                println!("Child exited (status {})", exit_code);
                                self.inferior = None;
                            },
                            Status::Signaled(signal) => {
                                println!("Child exited due to signal {}", signal);
                                self.inferior = None;
                            },
                            Status::Stopped(signal, rip) => {
                                println!("Child stopped (signal {})", signal);
                                println!("Stopped at {}", self.debug_data.get_line_from_addr(rip).unwrap());
                            },
                        }
                    }
                }
                DebuggerCommand::Backtrace => {
                    self.inferior.as_mut().unwrap().print_backtrace(&self.debug_data).unwrap();
                }
                DebuggerCommand::Breakpoint(addr) => {
                    let location;
                    if addr.starts_with("*") {
                        if let Some(address) = parse_address(&addr[1..]) {
                            location = address;
                        } else {
                            println!("Invalid breakpoint address.");
                            return;
                        }
                    } else if let Some(line) = usize::from_str_radix(&addr, 10).ok() {
                        // 猜测是本机环境问题导致无法使用行号设断点 
                        if let Some(address) = self.debug_data.get_addr_for_line(None, line) {
                            location = address;
                        } else {
                            println!("Invalid breakpoint line.");
                            return;
                        }
                    } else if let Some(address) = self.debug_data.get_addr_for_function(None, &addr) {
                        location = address;
                    } else {
                        return;
                    }
                    
                    if self.inferior.is_some() {
                        if let Ok(orig_byte) = self.inferior.as_mut().unwrap().write_byte(location, 0xcc){
                            self.breakpoints_map.insert(location, orig_byte);
                        } else {
                            println!("Change instruction error.");
                            return ;
                        }
                    } else {   
                        // the inferior is none and it will be init when be created
                        self.breakpoints_map.insert(location, 0);
                    }
                    println!("Set breakpoint {} at {:#x}", self.breakpoints_map.len()-1, location);
                }
            }
        }
    }

    /// This function prompts the user to enter a command, and continues re-prompting until the user
    /// enters a valid command. It uses DebuggerCommand::from_tokens to do the command parsing.
    ///
    /// You don't need to read, understand, or modify this function.
    fn get_next_command(&mut self) -> DebuggerCommand {
        loop {
            // Print prompt and get next line of user input
            match self.readline.readline("(deet) ") {
                Err(ReadlineError::Interrupted) => {
                    // User pressed ctrl+c. We're going to ignore it
                    println!("Type \"quit\" to exit");
                }
                Err(ReadlineError::Eof) => {
                    // User pressed ctrl+d, which is the equivalent of "quit" for our purposes
                    return DebuggerCommand::Quit;
                }
                Err(err) => {
                    panic!("Unexpected I/O error: {:?}", err);
                }
                Ok(line) => {
                    if line.trim().len() == 0 {
                        continue;
                    }
                    self.readline.add_history_entry(line.as_str());
                    if let Err(err) = self.readline.save_history(&self.history_path) {
                        println!(
                            "Warning: failed to save history file at {}: {}",
                            self.history_path, err
                        );
                    }
                    let tokens: Vec<&str> = line.split_whitespace().collect();
                    if let Some(cmd) = DebuggerCommand::from_tokens(&tokens) {
                        return cmd;
                    } else {
                        println!("Unrecognized command.");
                    }
                }
            }
        }
    }
}
