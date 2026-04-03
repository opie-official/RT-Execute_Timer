pub mod build_command;
pub mod execute;
pub mod types;

use crate::execute::make_keys;
use build_command::build_configurations;
use execute::spawn;
use std::env;
use types::{Configuration};

fn print_help() {
    println!("Usage: rt <-keys> -c=\"<command with args>\"");
    println!("Keys:");
    println!("-min                          -- turns of stdout and stderr of process");
    println!(
        "-err                          -- turns of stdout of process only (stdout turning on)"
    );
    println!(
        "-time=<val>                   -- sets a measure of execution time (can be ms, s, m, h)"
    );
    println!(
        "-proc=<val>                   -- sets the command interpreter of process (can be bash, sh, zsh, cmd, powershell)"
    );
    println!(
        "-test_time=<oper:val:measure> -- Tests the execution time. The first parameter specifies the comparison operation (<, <=, >, >=, ==, !=); The second parameter is the value against which the execution time is compared; The third parameter specifies the unit of measurement for the second parameter."
    );
    println!(
        "-c=<\"val\">                  -- Command that will be executed and their execution time will be measured"
    );
    println!(
        "--                            -- Separates global keys (which apply to all commands) from local keys (Optional if there are no global keys)"
    );
    println!(
        "\nExamples:\n\
     rt -time=\"s\" -- -c=\"echo Hello\" -c=\"mkdir dir\"\n\
     rt -c=\"touch file.txt\"\n\
     rt -proc=\"sh\" -c=\"yes \"yes, this is a string\" | head -n 10\""
    );
}

fn parse_args(args: &Vec<String>) {
    let mut configurations: Vec<Configuration> = Vec::new();
    let mut glob_args: Vec<Vec<String>>=Vec::new();
    build_configurations(&args, &mut configurations, &mut glob_args);
    println!("start executing...");
    let global = make_keys(&glob_args);

    for i in &configurations {
        spawn(&i, &global);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => print_help(),
        _ => parse_args(&args),
    }
}
