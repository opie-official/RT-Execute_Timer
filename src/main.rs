use std::env;
use std::process::{Command, Stdio};
use std::time::Instant;

fn print_help() {
    println!("Usage: rt -c=\"<command with args>\"");
}
#[derive(Debug, Clone, PartialEq)]
struct Configuration {
    options: Vec<Vec<String>>,
    command: String,
}

impl Configuration {
    fn new() -> Configuration {
        Self {
            options: Vec::new(),
            command: String::new(),
        }
    }
    fn clear(&mut self) {
        self.options.clear();
        self.command.clear();
    }
}

#[derive(Clone)]
enum GeneralOption<T> where T: Clone {
    None,
    Some(T),
}

const SEPARATOR: &str = "______________________________________________";
const SEPARATOR2: &str = "-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-";

fn spawn(conf: &Configuration, time_m: &Vec<&str>, proc_m: &Vec<(&str, &str)>,
         min: &GeneralOption<()>,
         err: &GeneralOption<()>,
         proc: &GeneralOption<String>,
         time: &GeneralOption<String>) {
    println!("{SEPARATOR}\nCommand \"{}\"", conf.command);
    let mut min: GeneralOption<()> = min.clone();
    let mut err: GeneralOption<()> = err.clone();
    let mut proc: GeneralOption<String> = proc.clone();
    let mut time: GeneralOption<String> = time.clone();
    // let mut test_time: GeneralOption<(String, i32, String)> = GeneralOption::None;
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
        } /*else if i.contains(&"-test-time".to_string()) {
            if i.len() >= 4 {
                if i[2].parse::<i32>().is_ok() {
                    test_time =
                        GeneralOption::Some((i[1].clone(), i[2].parse().unwrap(), i[3].clone()));
                }
            }
        }*/
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

    /*match &test_time {
        GeneralOption::Some((op, _, measure)) => {
            if !op_m.contains(&op.as_str()) {
                println!("incorrect operator: {}", op);
                return;
            }
            if !time_m.contains(&measure.as_str()) {
                println!("incorrect time measure: {}", measure);
                return;
            }
        }
        _ => (),
    }*/

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
}

fn parse_args(args: &Vec<String>) {
    let mut configurations: Vec<Configuration> = Vec::new();
    let mut glob_args: Vec<Vec<String>> = Vec::new();
    build(&args, &mut configurations, &mut glob_args);
    println!("start executing...\n{SEPARATOR}");

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
    }

    for i in &configurations {
        spawn(&i, &time_measure, &proc_measure, &min, &err, &proc, &time);
    }
}

fn parse_arg(container: &mut Vec<String>, i: &String, keys: &Vec<&str>) -> bool {
    let index = i.find("=");
    if index.is_some() {
        let first = &i[..index.unwrap()];
        let second = &i[index.unwrap() + 1..];
        if !keys.contains(&first.trim()) {
            println!("Key {first} doesn`t exist. Skip");
            return false;
        }
        if second.len() == 0 {
            println!("Value of key {first} is empty. Skip");
            return false;
        }
        if second.trim().contains(":") {
            let splited: Vec<&str> = second.trim().split(":").collect();
            container.push(first.trim().to_string());
            for i in splited.iter() {
                container.push(i.trim().to_string());
            }
        } else {
            container.push(first.trim().to_string());
            container.push(second.trim().to_string());
        }
    } else {
        container.push(i.trim().to_string());
    }
    true
}

fn build(
    args: &Vec<String>,
    configurations: &mut Vec<Configuration>,
    glob_args: &mut Vec<Vec<String>>,
) {
    let keys: Vec<&str> = vec![
        "-f",
        "-proc",
        "-time",
        "-min",
        "-err",
        "--",
        "-c",
    ];

    let mut current: Configuration = Configuration::new();
    let mut is_glob = true;
    for i in &args[1..] {
        if is_glob &&i == "--" {
            is_glob = false;
        }
        if !is_glob &&i == "--" {
            continue;
        }
        if is_glob {
            let mut _arg: Vec<String> = Vec::new();
            let res = parse_arg(&mut _arg, &i, &keys);
            if res {
                glob_args.push(_arg);
            }
        } else {
            let mut _arg: Vec<String> = Vec::new();
            let res = parse_arg(&mut _arg, &i, &keys);

            if res {
                if i.starts_with("-c") {
                    if _arg.len() == 2 {
                        current.command = _arg[1].clone();
                        configurations.push(current.clone());
                        current.clear();
                    } else {
                        println!("Invalid option {i}");
                        current.clear();
                        continue;
                    }
                } else {
                    current.options.push(_arg);
                }
            }
        }
    }

    if !current.command.is_empty() {
        configurations.push(current);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => print_help(),
        _ => parse_args(&args),
    }
}
