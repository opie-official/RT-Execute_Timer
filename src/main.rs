pub mod build_command;
pub mod types;

use build_command::build_configurations;
use std::collections::HashMap;
use std::env;
use std::process::{Command, Stdio};
use std::time::Instant;
use types::{Configuration, GeneralOption, TimeOrdering};
const SEPARATOR: &str = "______________________________________________";
const SEPARATOR2: &str = "-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-";

fn print_help() {

    println!("Usage: rt <-keys> -c=\"<command with args>\"");
    println!("Keys:");
    println!("-min                          -- turns of stdout and stderr of process");
    println!("-err                          -- turns of stdout of process only (stdout turning on)");
    println!("-time=<val>                   -- sets a measure of execution time (can be ms, s, m, h)");
    println!("-proc=<val>                   -- sets the command interpreter of process (can be bash, sh, zsh, cmd, powershell)");
    println!("-test_time=<oper:val:measure> -- Tests the execution time. The first parameter specifies the comparison operation (<, <=, >, >=, ==, !=); The second parameter is the value against which the execution time is compared; The third parameter specifies the unit of measurement for the second parameter.");
    println!("-c=<\"val\">                  -- Command that will be executed and their execution time will be measured");
    println!("--                            -- Separates global keys (which apply to all commands) from local keys (Optional if there are no global keys)");
    println!("\nExamples:\n\
     rt -time=\"s\" -- -c=\"echo Hello\" -c=\"mkdir dir\"\n\
     rt -c=\"touch file.txt\"\n\
     rt -proc=\"sh\" -c=\"yes \"yes, this is a string\" | head -n 10\"");
}

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
    if from=="ms"{
        return time/map[to];
    }
    let in_ms = time * map[from];
    in_ms / map[to]
}

fn spawn(
    conf: &Configuration,
    time_m: &Vec<&str>,
    proc_m: &Vec<(&str, &str)>,
    min: &GeneralOption<()>,
    err: &GeneralOption<()>,
    proc: &GeneralOption<String>,
    time: &GeneralOption<String>,
    test_time: &GeneralOption<(TimeOrdering, f64, String)>,
) {
    println!("{SEPARATOR}\nCommand \"{}\"", conf.command);
    let mut min: GeneralOption<()> = min.clone();
    let mut err: GeneralOption<()> = err.clone();
    let mut proc: GeneralOption<String> = proc.clone();
    let mut time: GeneralOption<String> = time.clone();
    let mut test_time: GeneralOption<(TimeOrdering, f64, String)> = test_time.clone();
    for i in conf.options.iter() {
        if i.contains(&"-min".to_string()) {
            min = GeneralOption::Some(());
        } else if i.contains(&"-err".to_string()) {
            err = GeneralOption::Some(());
        } else if i.contains(&"-time".to_string()) {
            if i.len() >= 2 {
                time = GeneralOption::Some(i[1].clone());
            }
        } else if i.contains(&"-proc".to_string()) {
            if i.len() >= 2 {
                proc = GeneralOption::Some(i[1].clone());
            }
        } else if i.contains(&"-test-time".to_string()) {
            if i.len() >= 4 {
                match i[3].as_str() {
                    "ms"=>(),
                    "s"=>(),
                    "m"=>(),
                    "h"=>(),
                    _=>{
                        println!("Invalid time measure {}. Skipping", i[3]);
                        return;
                    }
                }
                if i[2].parse::<f64>().is_ok() {
                    test_time = match i[1].as_str() {
                        ">" => GeneralOption::Some((
                            TimeOrdering::GT,
                            i[2].parse().unwrap(),
                            i[3].clone(),
                        )),
                        "<" => GeneralOption::Some((
                            TimeOrdering::LT,
                            i[2].parse().unwrap(),
                            i[3].clone(),
                        )),
                        ">=" => GeneralOption::Some((
                            TimeOrdering::GE,
                            i[2].parse().unwrap(),
                            i[3].clone(),
                        )),
                        "<=" => GeneralOption::Some((
                            TimeOrdering::LE,
                            i[2].parse().unwrap(),
                            i[3].clone(),
                        )),
                        "==" => GeneralOption::Some((
                            TimeOrdering::EQ,
                            i[2].parse().unwrap(),
                            i[3].clone(),
                        )),
                        "!=" => GeneralOption::Some((
                            TimeOrdering::NE,
                            i[2].parse().unwrap(),
                            i[3].clone(),
                        )),
                        _ => GeneralOption::Some((
                            TimeOrdering::EQ,
                            i[2].parse().unwrap(),
                            i[3].clone(),
                        )),
                    };
                }
            }
        }
    }

    match &proc {
        GeneralOption::Some(_proc) => {
            let mut flag = false;
            for (v, _) in proc_m {
                if *v == _proc {
                    flag = true;
                }
            }
            if !flag {
                println!("incorrect command interpreter: {}", _proc);
                return;
            }
        }
        _ => (),
    }
    match &time {
        GeneralOption::Some(_time) => {
            if !time_m.contains(&_time.as_str()) {
                println!("incorrect time measure: {}", _time);
                return;
            }
        }
        _ => (),
    }

    let mut command: Command = Command::new("");
    if cfg!(target_os = "windows") {
        if let GeneralOption::Some(v) = proc {
            let windows = vec!["cmd", "powershell"];
            if !windows.contains(&v.as_str()) {
                println!("Incorrect command interpreter: {v}, switch to cmd");
                command = Command::new("cmd");
                command.arg("/C");
            } else {
                let mut flag = false;
                for (_v, k) in proc_m.iter() {
                    if &v.as_str() == _v {
                        command = Command::new(&v);
                        command.arg(&k);
                        flag = true;
                        break;
                    }
                }
                if !flag {
                    println!("Incorrect command interpreter: {v}, switch to cmd");
                    command = Command::new("cmd");
                    command.arg("/C");
                }
            }
        } else {
            command = Command::new("cmd");
            command.arg("/C");
        }
    } else {
        if let GeneralOption::Some(v) = proc {
            let unix = vec!["bash", "sh", "zsh"];
            if !unix.contains(&v.as_str()) {
                println!("Incorrect command interpreter: {v}, switch to sh");
                command = Command::new("sh");
                command.arg("-c");
            } else {
                let mut flag = false;
                for (_v, k) in proc_m.iter() {
                    if &v.as_str() == _v {
                        command = Command::new(&v);
                        command.arg(&k);
                        flag = true;
                        break;
                    }
                }
                if !flag {
                    println!("Incorrect command interpreter: {v}, switch to sh");
                    command = Command::new("sh");
                    command.arg("-c");
                }
            }
        } else {
            command = Command::new("sh");
            command.arg("-c");
        }
    }

    if let GeneralOption::Some(()) = min {
        command.stdout(Stdio::null());
        command.stderr(Stdio::null());
    } else if let GeneralOption::Some(()) = err {
        command.stdout(Stdio::null());
    } else {
        println!("output\n{SEPARATOR2}");
    }
    command.arg(&conf.command);
    let _time = Instant::now();
    let code = command.status().unwrap();
    let elapsed = _time.elapsed();

    let time_v = match &time {
        GeneralOption::Some(v) if v == "s" => elapsed.as_secs_f64(),
        GeneralOption::Some(v) if v == "m" => elapsed.as_secs_f64() / 60.0,
        GeneralOption::Some(v) if v == "h" => elapsed.as_secs_f64() / 3600.0,
        _ => elapsed.as_millis() as f64,
    };

    let measure_time = match &time {
        GeneralOption::Some(v) if v == "s" => "s",
        GeneralOption::Some(v) if v == "m" => "m",
        GeneralOption::Some(v) if v == "h" => "h",
        _ => "ms",
    };
    if let GeneralOption::None = min {
        println!("\n{SEPARATOR2}");
    } else if let GeneralOption::None = err {
        println!("\n{SEPARATOR2}");
    }

    println!(
        "Program \"{}\" executed at {time_v}{measure_time} with {code}",
        conf.command
    );

    match test_time{
        GeneralOption::None=>(),
        GeneralOption::Some((oper, val, measure))=>{
            let time_measure = match &time{
                GeneralOption::Some(v) => v,
                _=> "ms"
            };
            let time_parsed = parse_time(time_v, time_measure, measure.as_str());
            match &oper{
                TimeOrdering::GT=>{
                    let op = time_parsed > val;
                    if op{
                        println!("** Time test was passed! {time_parsed}{measure} > {val}{measure} ({} difference)", time_parsed-val);

                    }else{
                        println!("!! Time test was not passed! {time_parsed}{measure} <= {val}{measure} ({} difference)", val-time_parsed);
                    }
                },TimeOrdering::LT=>{
                    let op = time_parsed < val;
                    if op{
                        println!("** Time test was passed! {time_parsed}{measure} < {val}{measure} ({} difference)", -time_parsed+val);

                    }else{
                        println!("!! Time test was not passed! {time_parsed}{measure} >= {val}{measure} ({} difference)", -val+time_parsed);
                    }
                },TimeOrdering::GE=>{
                    let op = time_parsed >= val;
                    if op{
                        println!("** Time test was passed! {time_parsed}{measure} >= {val}{measure} ({} difference)", time_parsed-val);

                    }else{
                        println!("!! Time test was not passed! {time_parsed}{measure} < {val}{measure} ({} difference)", val-time_parsed);
                    }
                },TimeOrdering::LE=>{
                    let op = time_parsed <= val;
                    if op{
                        println!("** Time test was passed! {time_parsed}{measure} <= {val}{measure} ({} difference)", -time_parsed+val);

                    }else{
                        println!("!! Time test was not passed! {time_parsed}{measure} > {val}{measure} ({} difference)", -val+time_parsed);
                    }
                },TimeOrdering::EQ=>{
                    let op = time_parsed == val;
                    if op{
                        println!("** Time test was passed! {time_parsed}{measure} == {val}{measure}");

                    }else{
                        println!("!! Time test was not passed! {time_parsed}{measure} != {val}{measure} ({} difference)", time_parsed-val);
                    }
                },TimeOrdering::NE=>{
                    let op = time_parsed != val;
                    if op{
                        println!("** Time test was passed! {time_parsed}{measure} != {val}{measure} ({} difference)", time_parsed-val);

                    }else{
                        println!("!! Time test was not passed! {time_parsed}{measure} == {val}{measure}");
                    }
                },
            }
        }
    }
}

fn parse_args(args: &Vec<String>) {
    let mut configurations: Vec<Configuration> = Vec::new();
    let mut glob_args: Vec<Vec<String>> = Vec::new();
    build_configurations(&args, &mut configurations, &mut glob_args);
    println!("start executing...");

    let time_measure = vec!["ms", "s", "m", "h"];
    let proc_measure = vec![
        ("bash", "-c"),
        ("cmd", "/C"),
        ("powershell", "-Command"),
        ("sh", "-c"),
        ("zsh", "-c"),
    ];

    //let op_measure = vec!["<", ">", "<=", ">=", "==", "!="];

    let mut min: GeneralOption<()> = GeneralOption::None;
    let mut err: GeneralOption<()> = GeneralOption::None;
    let mut proc: GeneralOption<String> = GeneralOption::None;
    let mut time: GeneralOption<String> = GeneralOption::None;
    let mut test_time: GeneralOption<(TimeOrdering, f64, String)> = GeneralOption::None;

    for i in glob_args {
        if i.contains(&"-min".to_string()) {
            min = GeneralOption::Some(());
        }
        if i.contains(&"-err".to_string()) {
            err = GeneralOption::Some(());
        }
        if i.contains(&"-proc".to_string()) {
            if i.len() == 2 {
                proc = GeneralOption::Some(i[1].clone());
            }
        }
        if i.contains(&"-time".to_string()) {
            if i.len() == 2 {
                time = GeneralOption::Some(i[1].clone());
            }
        }
        if i.contains(&"-test-time".to_string()) {
            if i.len() == 3 {
                let ordering = match i[0].as_str() {
                    "==" => TimeOrdering::EQ,
                    "!=" => TimeOrdering::NE,
                    "=>" => TimeOrdering::GE,
                    "<=" => TimeOrdering::LE,
                    "<" => TimeOrdering::LT,
                    ">" => TimeOrdering::GT,
                    _ => TimeOrdering::EQ,
                };
                let time_ = i[1].parse::<f64>();
                if time_.is_ok() {
                    let measure_ = i[2].clone();
                    if ["ms", "s", "m", "h"].contains(&measure_.as_str()) {
                        test_time = GeneralOption::Some((ordering, time_.unwrap(), measure_));
                    }
                }
            }
        }
    }

    for i in &configurations {
        spawn(
            &i,
            &time_measure,
            &proc_measure,
            &min,
            &err,
            &proc,
            &time,
            &test_time,
        );
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => print_help(),
        _ => parse_args(&args),
    }
}
