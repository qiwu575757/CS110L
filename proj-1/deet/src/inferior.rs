use nix::sys::ptrace;
use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use std::borrow::Borrow;
use std::os::unix::process::CommandExt;
use std::process::Child;
use std::process::Command;
use crate::dwarf_data::{DwarfData, Error as DwarfError};
use std::mem::size_of;
use std::collections::HashMap;

pub enum Status {
    /// Indicates inferior stopped. Contains the signal that stopped the process, as well as the
    /// current instruction pointer that it is stopped at.
    Stopped(signal::Signal, usize),
    
    /// Indicates inferior exited normally. Contains the exit status code.
    Exited(i32),

    /// Indicates the inferior exited due to a signal. Contains the signal that killed the
    /// process.
    Signaled(signal::Signal),
}

/// This function calls ptrace with PTRACE_TRACEME to enable debugging on a process. You should use
/// pre_exec with Command to call this in the child process.
fn child_traceme() -> Result<(), std::io::Error> {
    ptrace::traceme().or(Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "ptrace TRACEME failed",
    )))
}

fn align_addr_to_word(addr: usize) -> usize {
    addr & (-(size_of::<usize>() as isize) as usize)
}

pub struct Inferior {
    child: Child,
}

impl Inferior {
    /// Attempts to start a new inferior process. Returns Some(Inferior) if successful, or None if
    /// an error is encountered.
    pub fn new(target: &str, args: &Vec<String>, breakpoints_map: &mut HashMap<usize, u8>) -> Option<Inferior> {
        let mut cmd = Command::new(target);
        cmd.args(args);
        unsafe {
            cmd.pre_exec(child_traceme);    
        }
        let child = cmd.spawn().ok()?;
        
        let mut inferior = Inferior{child : child};
        for br in breakpoints_map {
            match inferior.write_byte(*(br.0), 0xcc) {
                Ok(orig_byte) => {
                    *(br.1) = orig_byte;
                    continue;
                },
                Err(_) => {
                    println!("Set breakpoint error at address {:#x}", br.0);
                    break;
                }
            }
        }

        Some(inferior)
    }

    /// Returns the pid of this inferior.
    pub fn pid(&self) -> Pid {
        nix::unistd::Pid::from_raw(self.child.id() as i32)
    }

    /// Calls waitpid on this inferior and returns a Status to indicate the state of the process
    /// after the waitpid call.
    pub fn wait(&self, options: Option<WaitPidFlag>) -> Result<Status, nix::Error> {
        Ok(match waitpid(self.pid(), options)? {
            WaitStatus::Exited(_pid, exit_code) => Status::Exited(exit_code),
            WaitStatus::Signaled(_pid, signal, _core_dumped) => Status::Signaled(signal),
            WaitStatus::Stopped(_pid, signal) => {
                let regs = ptrace::getregs(self.pid())?;
                Status::Stopped(signal, regs.rip as usize)
            }
            other => panic!("waitpid returned unexpected status: {:?}", other),
        })
    }


    pub fn continue_run(&mut self, signal: Option<signal::Signal>, breakpoints_map: &mut HashMap<usize, u8>) -> Result<Status, nix::Error> {
        let mut regs = ptrace::getregs(self.pid())?;
        let rip = regs.rip as usize;
        
        // when a inferior is stopped, there are 2 possibilities:
        // * stop by a breakpoint
        // * stop by crtl + c
        // test inferior is stopped at a breakpoint
        // inferior is stopped at a breakpoint
        if let Some(orig_byte) = breakpoints_map.get(&(rip-1)) {
            // restore the first byte of the instruction we replaced
            self.write_byte(rip-1, *orig_byte).unwrap();
            // set %rip = %rip - 1 to rewind the instruction pointer
            regs.rip = (rip - 1) as u64;
            ptrace::setregs(self.pid(), regs).unwrap();
            // ptrace::step to go to next instruction
            ptrace::step(self.pid(), signal)?;
            // wait for inferior to stop due to SIGTRAP
            match self.wait(None).unwrap() {
                Status::Exited(exit_code) => {
                    return Ok(Status::Exited(exit_code));
                },
                Status::Signaled(signal) => {
                    return Ok(Status::Signaled(signal));
                },
                Status::Stopped(signal, _) => {
                    if signal.eq(&(signal::Signal::SIGTRAP)) {
                        // restore 0xcc in the breakpoint location
                        println!("Stopped at a breakpoint.");
                        self.write_byte(rip-1, 0xcc).unwrap();
                    } else {
                        return Ok(Status::Stopped(signal, rip))
                    }
                },
            }
        }

        ptrace::cont(self.pid(), signal)?;
        self.wait(None)
    }
    
    pub fn kill(&mut self) {
        self.child.kill().unwrap();
        self.wait(None).unwrap();
        println!("Killing running inferior (pid {})", self.pid());
    }

    pub fn print_backtrace(&self, debug_data: &DwarfData) -> Result<(), nix::Error> {
        // instruction_ptr = %rip
        // base_ptr = %rbp
        // while true:
        //     print function/line number for instruction_ptr
        //     if function == "main":
        //         break
        //     instruction_ptr = read memory at (base_ptr + 8)
        //     base_ptr = read memory at base_ptr

        let regs = ptrace::getregs(self.pid())?;
        let mut rip = regs.rip as usize;
        let mut rbp = regs.rbp as usize;
        loop {
            let _function = debug_data.get_function_from_addr(rip).unwrap();
            let _line = debug_data.get_line_from_addr(rip).unwrap();
            println!("{} ({})", _function, _line);
            if _function == "main" {
                break;
            }
            rip = ptrace::read(self.pid(), (rbp+8) as ptrace::AddressType)? as usize;
            rbp = ptrace::read(self.pid(), rbp as ptrace::AddressType)? as usize;
        }

        Ok(())
    }

    pub fn write_byte(&mut self, addr: usize, val: u8) -> Result<u8, nix::Error> {
            let aligned_addr = align_addr_to_word(addr);
            let byte_offset = addr - aligned_addr;
            let word = ptrace::read(self.pid(), aligned_addr as ptrace::AddressType)? as u64;
            let orig_byte = (word >> 8 * byte_offset) & 0xff;
            let masked_word = word & !(0xff << 8 * byte_offset);
            let updated_word = masked_word | ((val as u64) << 8 * byte_offset);
            ptrace::write(
                self.pid(),
                aligned_addr as ptrace::AddressType,
                updated_word as *mut std::ffi::c_void,
            )?;
            Ok(orig_byte as u8)
    }
}
