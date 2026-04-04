use crate::conf::POSSIBLE;
use crate::types::{Configuration, Key, MyCommand};
use std::collections::HashMap;

// const POSSIBLE_OPERATIONS: &[&str] = &["<", ">", "<=", ">=", "==", "!="];
// const POSSIBLE_TIME_MEASURE: &[&str] = &["ms", "s", "m", "h"];


const DECIMAL:f64 = 100.0;

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
        return (time / map[to] * DECIMAL).round() / DECIMAL;
    }
    let in_ms = time * map[from];
    (in_ms / map[to] * DECIMAL).round() / DECIMAL
}
///
/// Parse memory from one measure to other
/// * mem - memory
/// * from - measure of mem
/// * to - measure that we want to get
///
/// returns: memory in other measure
///
fn parse_memory(mem: f64, from: &str, to: &str) -> f64 {
    if from == to {
        return mem;
    }
    let map: HashMap<&str, f64> = HashMap::from([
        ("bit", 8.0),
        ("b", 1024.0 * 8.0),
        ("kb", 1024.0 * 1024.0 * 8.0),
        ("mb", 1024.0 * 1024.0 * 1024.0 * 8.0),
        ("gb", 1024.0 * 1024.0 * 1024.0 * 1024.0 * 8.0),
    ]);

    if from == "bit" {
        return (mem / map[to] * DECIMAL).round() / DECIMAL;
    }
    let in_bit = mem * map[from];
    (in_bit / map[to] * DECIMAL).round() / DECIMAL
}

///
/// Make a process with command from configuration
/// * conf - configuration of one command with local keys
/// * global - global keys
///
/// returns ()
///
pub fn spawn(conf: &Configuration, global: &HashMap<String, Key>) {
    let mut keys = make_keys(&conf.options);

    rewrite_by_globals(&mut keys, &global);
    let cmd = build_command(&conf.command, &keys);
    if cmd.is_err() {
        return;
    }
    let mut command = cmd.unwrap();

    let one_line = match keys.get("-oneline") {
        Some(_) => true,
        _ => false,
    };
    let mut result = command.run(one_line); // running command with it output

    if let Some(val) = keys.get("-time") {
        if let Key::Value(val_) = val {
            let res = parse_time(result.time.0, result.time.1.as_str(), val_);
            result.time.0 = res;
            result.time.1 = val_.to_string();
        }
    }
    if let Some(val) = keys.get("-mem") {
        if let Key::Value(val_) = val {
            let peak = parse_memory(result.peak_mem.0, result.peak_mem.1.as_str(), val_);
            let sum = parse_memory(result.sum_mem.0, result.sum_mem.1.as_str(), val_);
            result.peak_mem = (peak, val_.to_string());
            result.sum_mem = (sum, val_.to_string());
        }
    }
    let mut test_time = false;
    let mut test_mem = false;
    if let Some(val_) = keys.get("-test_time") {
        if let Key::Three((op, value, measure)) = val_ {
            test_time = true;
            let new_time = parse_time(result.time.0, &result.time.1, measure);
            let res_comparing = match op.as_str() {
                "<" => new_time < *value,
                "<=" => new_time <= *value,
                ">" => new_time > *value,
                ">=" => new_time >= *value,
                "==" => new_time == *value,
                "!=" => new_time != *value,
                _ => new_time == *value,
            };
            result.test_time = Some((
                res_comparing,
                new_time,
                op.to_string(),
                *value,
                measure.to_string(),
            ));
        }
    }
    if let Some(val_) = keys.get("-test_mem") {
        if let Key::Three((op, value, measure)) = val_ {
            test_mem = true;
            let new_mem_sum = parse_memory(result.sum_mem.0, &result.sum_mem.1, measure);
            let res_comparing_sum = match op.as_str() {
                "<" => new_mem_sum < *value,
                "<=" => new_mem_sum <= *value,
                ">" => new_mem_sum > *value,
                ">=" => new_mem_sum >= *value,
                "==" => new_mem_sum == *value,
                "!=" => new_mem_sum != *value,
                _ => new_mem_sum == *value,
            };
            let new_mem_peak = parse_memory(result.peak_mem.0, &result.peak_mem.1, measure);
            let res_comparing_peak = match op.as_str() {
                "<" => new_mem_peak < *value,
                "<=" => new_mem_peak <= *value,
                ">" => new_mem_peak > *value,
                ">=" => new_mem_peak >= *value,
                "==" => new_mem_peak == *value,
                "!=" => new_mem_peak != *value,
                _ => new_mem_peak == *value,
            };

            result.test_mem_peak = Some((
                res_comparing_peak,
                new_mem_peak,
                op.to_string(),
                *value,
                measure.to_string(),
            ));
            result.test_mem_sum = Some((
                res_comparing_sum,
                new_mem_sum,
                op.to_string(),
                *value,
                measure.to_string(),
            ));
        }
    }
    if !one_line {
        let mem_peak = result.clone().sum_mem;
        let mem_sum = result.clone().peak_mem;
        let mut fmt = format!(
            "Program {} executed at {}{}; {}{} memory at peak, {}{} memory at sum with exit code {}",
            &conf.command,
            result.time.0,
            result.time.1,
            &mem_peak.0,
            &mem_peak.1,
            &mem_sum.0,
            &mem_sum.1,
            result.exit_code
        );
        if test_time {
            let unwrapped = result.clone().test_time.unwrap();
            fmt = format!(
                "{fmt}\n - test time: {}; {}{} {} {}{}",
                if unwrapped.0 == true { "OK" } else { "FAILED" },
                unwrapped.1,
                unwrapped.4,
                unwrapped.2,
                unwrapped.3,
                unwrapped.4
            );
        }
        if test_mem {
            let unwrapped_peak = result.clone().test_mem_peak.unwrap();
            let unwrapped_sum = result.clone().test_mem_sum.unwrap();
            fmt = format!(
                "{fmt}\n - test memory peak: {}; {}{} {} {}{}",
                if unwrapped_peak.0 == true {
                    "OK"
                } else {
                    "FAILED"
                },
                unwrapped_peak.1,
                unwrapped_peak.4,
                unwrapped_peak.2,
                unwrapped_peak.3,
                unwrapped_peak.4
            );
            fmt = format!(
                "{fmt}\n - test memory sum: {}; {}{} {} {}{}",
                if unwrapped_sum.0 == true {
                    "OK"
                } else {
                    "FAILED"
                },
                unwrapped_sum.1,
                unwrapped_sum.4,
                unwrapped_sum.2,
                unwrapped_sum.3,
                unwrapped_sum.4
            );
        }
        println!("{fmt}");
    } else {
        let mem_peak = result.clone().sum_mem;
        let mem_sum = result.clone().peak_mem;
        let mut fmt = format!(
            "{}: [time: {}{}; mem peak: {}{}; mem sum: {}{} ] -> {}",
            &conf.command,
            result.time.0,
            result.time.1,
            &mem_peak.0,
            &mem_peak.1,
            &mem_sum.0,
            &mem_sum.1,
            result.exit_code
        );
        if test_time {
            fmt = format!("{fmt}; test time: {}", result.test_time.unwrap().0);
        }
        if test_mem {
            fmt = format!(
                "{fmt}; test mem peak: {}; test mem sum: {}",
                result.test_mem_peak.unwrap().0,
                result.test_mem_sum.unwrap().0
            );
        }
        println!("{fmt}");
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
    let global_keys = globals.keys().collect::<Vec<_>>();
    for key in global_keys {
        locals.insert(key.to_string(), globals.get(key).unwrap().clone());
    }
}

///
/// Building a command object by local keys
/// * cmd - command name
/// * keys - local keys (rewrote)
///
/// returns Command if not there are any errors in parsing keys
///
fn build_command(cmd_: &String, keys: &HashMap<String, Key>) -> Result<MyCommand, String> {
    let mut cmd = MyCommand::new();
    if let Some(val) = keys.get("-proc") {
        if let Key::Value(val_) = val {
            if PROC.contains(&val_.as_str()) {
                let i = PROC.iter().position(|e| e == &val_).unwrap();
                let key_launch = PROC_KEYS[i];
                cmd.set_ci(&val_, &key_launch.to_string());
            } else {
                return Err(String::from("Invalid command interpreter. Skip"));
            }
        }
    } else {
        cmd.set_ci(&PROC_DEFAULT[0].to_string(), &PROC_DEFAULT[1].to_string());
    }
    if let Some(_) = keys.get("-min") {
        cmd.stdout_off();
        cmd.stderr_off();
    }
    if let Some(_) = keys.get("-err") {
        cmd.stdout_off();
    }
    if let Some(_) = keys.get("-oneline") {
        cmd.stdout_off();
        cmd.stderr_off();
    }
    cmd.set_command(&cmd_);
    Ok(cmd)
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
