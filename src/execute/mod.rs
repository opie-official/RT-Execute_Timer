use crate::types::{Configuration, Key};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::Instant;

const POSSIBLE: &[&str] = &[
    "-min",
    "-err",
    "-time",
    "-proc",
    "-mem",
    "-test_time",
    "-test_mem",
];
const POSSIBLE_OPERATIONS: &[&str] = &["<", ">", "<=", ">=", "==", "!="];
const POSSIBLE_TIME_MEASURE: &[&str] = &["ms", "s", "m", "h"];

const SEPARATOR: &str = "______________________________________________";
const SEPARATOR2: &str = "-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-";

#[cfg(target_os = "windows")]
const PROC: &[&str] = &["cmd", "powershell"];

#[cfg(target_os = "linux")]
const PROC: &[&str] = &["bash", "sh", "zsh"];
#[cfg(target_os = "macos")]
const PROC: &[&str] = &["sh", "bash", "zsh"];

#[cfg(target_os = "windows")]
const PROC_KEYS: &[&str] = &["/C", "-Command"];

#[cfg(target_os = "linux")]
const PROC_KEYS: &[&str] = &["-c", "-c", "-c"];
#[cfg(target_os = "macos")]
const PROC_KEYS: &[&str] = &["-c", "-c", "-c"];

#[cfg(target_os = "windows")]
const PROC_DEFAULT: &[&str] = &["cmd", "/C"];

#[cfg(target_os = "linux")]
const PROC_DEFAULT: &[&str] = &["bash", "-c"];

#[cfg(target_os = "macos")]
const PROC_DEFAULT: &[&str] = &["zsh", "-c"];


///
/// Parse time from one measure to other
/// * time - time
/// * from - measure of time
/// * to - measure that we want to get
///
/// returns: time in other measure
///
fn parse_time(time: f64, from: &str, to: &str) -> f64 {
    if from == to {
        return time;
    }
    let map: HashMap<&str, f64> = HashMap::from([
        ("ms", 1000.0),
        ("s", 60.0),
        ("m", 60.0 * 1000.0),
        ("h", 60.0 * 1000.0 * 60.0),
    ]);
    if from == "ms" {
        return time / map[to];
    }
    let in_ms = time * map[from];
    in_ms / map[to]
}

///
///Make a process with command from configuration
/// * conf - configuration of one command with local keys
/// * global - global keys
///
///returns ()
///
pub fn spawn(conf: &Configuration, global: &HashMap<String, Key>) {
    let mut keys = make_keys(&conf.options);

    rewrite_by_globals(&mut keys, &global);

    let cmd = build_command(&conf.command, &keys);
    if cmd.is_err() {
        println!("{SEPARATOR}Error: {}\n{SEPARATOR}", cmd.err().unwrap());
        return;
    }
    println!("{SEPARATOR}\nProgram \"{}\"", conf.command);
    println!("output\n{SEPARATOR2}");
    let mut command = cmd.unwrap();
    let start = Instant::now();
    let stat = command.status().unwrap();
    let elapsed = start.elapsed().as_millis() as f64;

    let mut times = 1;
    let mut time_measure = "ms";

    if let Some(val) = keys.get("-time") {
        if let Key::Value(val_) = val {
            match val_.as_str() {
                "s" => {
                    times = 1000;
                    time_measure = "s";
                }
                "m" => {
                    times = 60000;
                    time_measure = "m";
                }
                "h" => {
                    times = 60000 * 60;
                    time_measure = "h";
                }
                _ => (),
            }
        }
    }

    let finish_time = elapsed / times as f64;

    println!("{SEPARATOR2}");
    println!(
        "Program \"{}\" finished at {finish_time}{time_measure} with {stat}",
        conf.command
    );

    if let Some(val) = keys.get("-test_time") {
        test_time_manage(finish_time, &val, &time_measure);
    }
}

///
/// Test execution time by comparing it with some value
/// * finished_time - execution time
/// * val - configuration after -test_time=
/// * time_measure - time measure of finished_time
///
/// returns ()
///
fn test_time_manage(finished_time: f64, val: &Key, time_measure: &str) {
    if let Key::Three((op, val, me)) = val {
        if POSSIBLE_OPERATIONS.contains(&op.as_str())
            && POSSIBLE_TIME_MEASURE.contains(&me.as_str())
        {
            let parsed_time = parse_time(finished_time, time_measure, me);

            match op.as_str() {
                "<" => {
                    if parsed_time < *val {
                        println!(
                            "** Time test was passed!  {parsed_time}{me}<{val}{me}({} difference)",
                            -parsed_time + val
                        )
                    }else {
                        println!(
                            "!! Time test was not passed!  {parsed_time}{me}>={val}{me}({} difference)",
                            parsed_time - val
                        )
                    }
                },"<=" => {
                    if parsed_time <= *val {
                        println!(
                            "** Time test was passed!  {parsed_time}{me}<={val}{me}({} difference)",
                            -parsed_time + val
                        )
                    }else {
                        println!(
                            "!! Time test was not passed!  {parsed_time}{me}>{val}{me}({} difference)",
                            parsed_time - val
                        )
                    }
                },">" => {
                    if parsed_time > *val {
                        println!(
                            "** Time test was passed!  {parsed_time}{me}>{val}{me}({} difference)",
                            parsed_time - val
                        )
                    }else {
                        println!(
                            "!! Time test was not passed!  {parsed_time}{me}<={val}{me}({} difference)",
                            -parsed_time + val
                        )
                    }
                },">=" => {
                    if parsed_time >= *val {
                        println!(
                            "** Time test was passed!  {parsed_time}{me}>={val}{me}({} difference)",
                            parsed_time - val
                        )
                    }else {
                        println!(
                            "!! Time test was not passed!  {parsed_time}{me}<{val}{me}({} difference)",
                            -parsed_time + val
                        )
                    }
                },"==" => {
                    if parsed_time == *val {
                        println!(
                            "** Time test was passed!  {parsed_time}{me}=={val}{me}({} difference)",
                            parsed_time - val
                        )
                    }else {
                        println!(
                            "!! Time test was not passed!  {parsed_time}{me}!={val}{me}({} difference)",
                            parsed_time - val
                        )
                    }
                },"!=" => {
                    if parsed_time != *val {
                        println!(
                            "** Time test was passed!  {parsed_time}{me}!={val}{me}({} difference)",
                            parsed_time - val
                        )
                    }else {
                        println!(
                            "!! Time test was not passed!  {parsed_time}{me}=={val}{me}({} difference)",
                            parsed_time - val
                        )
                    }
                },
                _ => (),
            }
        }
    }
}

///
/// Rewrite local keys of configuration by global keys
/// * locals - local keys
/// * globals - global keys
///
/// returns () (changing locals in-place)
///
fn rewrite_by_globals(locals: &mut HashMap<String, Key>, globals: &HashMap<String, Key>) {
    let local_keys = locals.keys().cloned().collect::<Vec<String>>();
    for key in local_keys {
        if let Some(val) = globals.get(&key) {
            locals.insert(key.clone(), val.clone());
        }
    }
}

///
/// Building a command object by local keys
/// * cmd - command name
/// * keys - local keys (rewrote)
///
/// returns Command if not there are any errors in parsing keys
///
fn build_command(cmd: &String, keys: &HashMap<String, Key>) -> Result<Command, String> {
    let mut command = Command::new("");
    if let Some(val) = keys.get("-proc") {
        if let Key::Value(val_) = val {
            if PROC.contains(&val_.as_str()) {
                command = Command::new(&val_);
                let i = PROC.iter().position(|e| e == &val_).unwrap();
                let key_launch = PROC_KEYS[i];
                command.arg(&key_launch);
            } else {
                return Err(String::from("Invalid command interpreter. Skip"));
            }
        }
    } else {
        command = Command::new(PROC_DEFAULT[0]);
        command.arg(PROC_DEFAULT[1]);
    }
    if let Some(_) = keys.get("-min") {
        command.stdout(Stdio::null());
        command.stderr(Stdio::null());
    }
    if let Some(_) = keys.get("-err") {
        command.stdout(Stdio::null());
    }
    command.arg(&cmd);

    Ok(command)
}

///
/// Parse string keys in hashmap
/// * keys - raw keys
///
/// returns hash-map with keys
///
pub fn make_keys(keys: &[Vec<String>]) -> HashMap<String, Key> {
    let mut result: HashMap<String, Key> = HashMap::new();
    for key in keys {
        if key.len() == 1 {
            let val = &key[0];
            if POSSIBLE.contains(&val.as_str()) {
                result.insert(val.clone(), Key::Basic);
            }
        } else if key.len() == 2 {
            let key_ = &key[0];
            let val_ = &key[1];
            if POSSIBLE.contains(&key_.as_str()) {
                result.insert(key_.clone(), Key::Value(val_.clone()));
            }
        } else if key.len() == 4 {
            let key_ = &key[0];
            let val1_ = &key[1];
            let val2_ = &key[2];
            let val3_ = &key[3];
            if POSSIBLE.contains(&key_.as_str()) {
                let parse = val2_.parse::<f64>();
                if parse.is_ok() {
                    result.insert(
                        key_.clone(),
                        Key::Three((val1_.clone(), parse.unwrap(), val3_.clone())),
                    );
                }
            }
        } else {
            continue;
        }
    }
    result
}
