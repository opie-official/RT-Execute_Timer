use std::env;
use std::process::Command;
use std::time::Instant;

fn print_help() {
    println!("Usage: rt -c=\"<command with args>\"");
}

#[derive(Clone, Debug)]
struct Conf {
    options: Vec<String>, // -f (full info)
    command: String,
}

impl Conf {
    fn new() -> Conf {
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
/// Parse input and build configurations with options, commands, and their args
fn build_configurations(args: &Vec<String>) -> Vec<Conf> {
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
            if cmd.is_empty() {
                panic!("Invalid empty option {i}");
            }

            let raw_command = cmd.to_string().clone();
            current.command = raw_command;

            configurations.push(current.clone());
            current.clear();
        }
    }

    configurations
}

const SEPARATOR: &str = "______________________________________________";
const SEPARATOR2: &str = "-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-";
fn parse_args(args: &Vec<String>) {
    let configurations = build_configurations(&args);
    println!("start executing...\n{SEPARATOR}");
    for i in configurations.iter() {
        let command = i.command.clone();

        println!("\n{SEPARATOR}\ncommand: {command}\n{SEPARATOR2}");
        if cfg!(target_os = "windows") {
            let start = Instant::now();
            let code = Command::new("cmd")
                .arg("/C")
                .arg(&command)
                .status()
                .expect("failed to execute process");
            let duration = start.elapsed();
            println!("\n{SEPARATOR}\n");
            println!(
                "Program \"{command}\" executed at {}ms with exit code {}",
                duration.as_millis(),
                code.code().unwrap()
            );
        } else {
            let start = Instant::now();
            let code = Command::new("sh")
                .arg("-c")
                .arg(&command)
                .status()
                .expect("failed to execute process");
            let duration = start.elapsed();
            println!("\n{SEPARATOR}\n");
            println!(
                "Program \"{command}\" executed at {}ms with exit code {}",
                duration.as_millis(),
                code.code().unwrap()
            );
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => print_help(),
        _ => parse_args(&args),
    }
}
