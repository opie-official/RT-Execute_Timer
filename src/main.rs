use std::env;
use std::process::Command;
use std::time::Instant;

fn print_help() {}

#[derive(Clone, Debug)]
struct Conf {
    options: Vec<String>, // -f (full info)
    command: String,
    args: Vec<String>,
}

impl Conf {
    fn new() -> Conf {
        Self {
            options: Vec::new(),
            command: String::new(),
            args: Vec::new(),
        }
    }
    fn clear(&mut self) {
        self.options.clear();
        self.command.clear();
        self.args.clear();
    }
}
/// Parse input and build configurations with options, commands, and their args
fn build_configurations(args: &Vec<String>) -> Vec<Conf>{
    let mut configurations: Vec<Conf> = Vec::new();
    let mut current: Conf = Conf::new();
    for i in args {
        if i.trim().starts_with("-") && !i.trim().starts_with("-c") {
            current.options.push(i.trim().to_string());
        } else if i.trim().starts_with("-c") {
            let index = i.find("=");
            if index.is_none() {
                panic!("Invalid option {i}");
            }
            let cmd = &i[index.unwrap() + 1..];
            if cmd.is_empty(){
                panic!("Invalid empty option {i}");
            }

            let raw_command = cmd.to_string().clone();

            let mut raw_string = String::new();
            let mut quotes: Vec<char> = Vec::new();
            let mut raw_list: Vec<String> = Vec::new();

            for ch in raw_command.chars() {
                // parse command by cmd-name and args-list
                // println!("char: {ch} {} : {}", ch == '\'' || ch == '"', raw_string);
                if ch == '\'' || ch == '"' {
                    // println!("quote taken {ch}");
                    if quotes.len() == 0 {
                        quotes.push(ch.clone());
                    } else {
                        let last = quotes.last().unwrap();
                        if *last == ch {
                            quotes.pop();
                        } else {
                            quotes.push(ch.clone());
                        }
                    }
                } else if ch != ' ' || (ch == ' ' && quotes.len() != 0) {
                    raw_string.push(ch.clone());
                } else if ch == ' ' && quotes.len() == 0 {
                    raw_list.push(raw_string.clone());
                    raw_string.clear();
                }
            }
            if !raw_string.is_empty() {
                raw_list.push(raw_string.clone());
            }
            println!("{raw_list:?}");
            let cmd = raw_list[0].trim().to_string();
            current.command = cmd.clone();
            if raw_list.len() >1 {
                let args = raw_list[1..].to_vec();
                current.args = args.clone();
            }
            configurations.push(current.clone());
            current.clear();
        }
    }

    // println!("configurations:\n{configurations:?}");
    configurations
}

const SEPARATOR: &str="______________________________________________";
fn parse_args(args: &Vec<String>) {
    let configurations = build_configurations(&args);
    println!("start executing...\n{SEPARATOR}");
    for i in configurations.iter() {
        let command = i.command.clone();
        let args = i.args.clone();
        let start = Instant::now();
        let code = Command::new(&command).args(&args).status().expect("failed to execute process");
        let duration = start.elapsed();
        println!("\n{SEPARATOR}\n");
        println!("Program \"{command}[{}]\" executed at {}ms with exit code {}", args.join(", "),duration.as_millis(), code.code().unwrap());


    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => print_help(),
        _ => parse_args(&args),
    }
}
