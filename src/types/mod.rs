use crate::conf::{SEPARATOR, SEPARATOR2};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

///
/// Raw representation of command configuration
/// * options - raw local keys
/// * command - command
///
/// Derives:
/// - Debug
/// - Clone
/// - PartialEq
///
#[derive(Debug, Clone, PartialEq)]
pub struct Configuration {
    pub options: Vec<Vec<String>>,
    pub command: String,
}

impl Configuration {
    ///
    /// Make object
    ///
    pub fn new() -> Configuration {
        Self {
            options: Vec::new(),
            command: String::new(),
        }
    }

    ///
    /// Clearing object fields
    ///
    pub fn clear(&mut self) {
        self.options.clear();
        self.command.clear();
    }
}

///
/// Representation of key`s value
///
/// Derives:
/// - Clone
/// - Debug
///
#[derive(Clone, Debug)]
pub enum Key {
    None,
    Basic,
    Value(String),
    Three((String, f64, String)),
}


///
/// Result of the program execution
/// - command: Command with it args
/// - time: elapsed time
/// - peak_mem: maximum memory usage by program
/// - sum_mem: summary memory usage by program
/// - test_time: test time information
/// - test_mem_sum: test mem_sum information
/// - test_mem_peak: test mem_peak information
/// - one_line: value of the key -oneline
/// - exit code: exit code of the program
///
/// Derives
/// - Clone
/// - Debug
///
#[derive(Clone, Debug)]
pub struct ProcessResult {
    pub command: String,
    pub time: (f64, String),
    pub peak_mem: (f64, String),
    pub sum_mem: (f64, String),
    pub test_time: Option<(bool, f64, String, f64, String)>,
    pub test_mem_sum: Option<(bool, f64, String, f64, String)>,
    pub test_mem_peak: Option<(bool, f64, String, f64, String)>,
    pub one_line: bool,
    pub exit_code: i32,
}

impl ProcessResult {
    ///
    /// Make object
    ///
    pub fn new() -> ProcessResult {
        Self {
            command: String::new(),
            time: (0.0, String::from("ms")),
            peak_mem: (0.0, String::from("kb")),
            sum_mem: (0.0, String::from("kb")),
            test_time: None,
            test_mem_sum: None,
            test_mem_peak: None,
            one_line: false,
            exit_code: 0,
        }
    }
}


///
/// Wrapper of process:Command
///
/// Derives:
/// - Debug
///
#[derive(Debug)]
pub struct MyCommand {
    pub command: Command,
}

impl MyCommand {
    ///
    /// Make Object
    ///
    pub fn new() -> MyCommand {
        Self {
            command: Command::new(""),
        }
    }
    ///
    /// Set command as command argument
    /// - cmd - command string
    ///
    /// returns &mut self
    ///
    pub fn set_command(&mut self, cmd: &String) -> &mut Self {
        self.command.arg(cmd);
        self
    }
    ///
    /// turns off the stdout
    ///
    pub fn stdout_off(&mut self) -> &mut Self {
        self.command.stdout(Stdio::null());
        self
    }
    ///
    /// turns off the stderr
    ///
    pub fn stderr_off(&mut self) -> &mut Self {
        self.command.stderr(Stdio::null());
        self
    }
    ///
    /// turns off the stdin
    ///
    pub fn stdin_off(&mut self) -> &mut Self {
        self.command.stdin(Stdio::null());
        self
    }
    
    ///
    /// Sets the command interpreter as command and their key as command argument
    /// 
    pub fn set_ci(&mut self, ci: &String, key: &String) -> &mut Self {
        self.command = Command::new(&ci);
        self.command.arg(&key);
        self
    }

    ///
    /// Running the command; measures their elapsed time and memory (in peak and summary)
    /// - one_line - key -oneline (needed for print in cli)
    /// 
    pub fn run(&mut self, one_line: bool) -> ProcessResult {
        let mut cmd = self.command.spawn().unwrap();
        let pid = Pid::from(cmd.id() as usize);

        let running_arg = Arc::new(AtomicBool::new(true));
        let running_clone = running_arg.clone();

        let peak_mem = Arc::new(Mutex::new(0u64));
        let peak_mem_clone = peak_mem.clone();
        let sum_mem = Arc::new(Mutex::new(0u64));
        let sum_mem_clone = sum_mem.clone();

        if !one_line {
            let args: Vec<&str> = self
                .command
                .get_args()
                .map(|e| e.to_str().unwrap())
                .collect();
            let cmd_ = args[args.len() - 1];
            println!("{SEPARATOR}\nCommand {}\noutput\n{SEPARATOR2}", cmd_);
        }

        let monitor = thread::spawn(move || {
            let mut system = System::new();
            while running_clone.load(Ordering::Relaxed) {
                let kind = ProcessRefreshKind::nothing().with_memory().without_tasks();
                system.refresh_processes_specifics(ProcessesToUpdate::Some(&[pid]), true, kind);
                #[cfg(not(target_os = "windows"))]
                {
                    if let Some(proc) = system.process(pid) {
                        let mem = proc.memory();
                        if mem < 10_000_000 {
                            let mut sum = sum_mem_clone.lock().unwrap();
                            let mut peak = peak_mem_clone.lock().unwrap();
                            if *peak < mem {
                                *peak = mem;
                            }
                            *sum += mem;
                        }
                    } else {
                        break;
                    }
                }
                #[cfg(target_os = "windows")]
                {
                    if let Some(mem)=measure_memory(pid.as_u32()){
                        if mem < 10_000_000 {
                            let mut sum = sum_mem_clone.lock().unwrap();
                            let mut peak = peak_mem_clone.lock().unwrap();

                            if *peak < mem {
                                *peak = mem;
                            }
                            *sum += mem;
                        }
                    }
                }
                thread::sleep(Duration::from_millis(10));
            }
        });

        let start = Instant::now();
        let code = cmd.wait().unwrap();
        let elapsed = start.elapsed().as_millis();
        running_arg.store(false, Ordering::Relaxed);
        monitor.join().unwrap();

        if !one_line {
            println!("{SEPARATOR2}");
        }

        let mut result = ProcessResult::new();
        let peak_memory = *peak_mem.lock().unwrap();
        let sum_memory = *sum_mem.lock().unwrap();

        result.time = (elapsed as f64, "ms".to_string());
        result.peak_mem = (peak_memory as f64, "kb".to_string());
        result.sum_mem = (sum_memory as f64, "kb".to_string());
        result.exit_code = code.code().unwrap();

        result
    }
}


///
/// Measures memory on Windows
/// 
#[cfg(target_os = "windows")]
fn measure_memory(pid: u32) ->Option<u64>{
    use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
    use windows::Win32::Foundation::CloseHandle;
    unsafe {
        let handle = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            false,
            pid
        ).ok();
        if let Some(handle) = handle {
            let mut counters = PROCESS_MEMORY_COUNTERS::default();
            let result = GetProcessMemoryInfo(
                handle, &mut counters, size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            );
            if result.is_ok(){
               let _ = CloseHandle(handle);

                return Some(counters.WorkingSetSize as u64/1024);
            }
            let _ = CloseHandle(handle);

        }
    }
    None
}