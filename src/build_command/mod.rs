use crate::types::Configuration;

///
/// Parses argument and build configurations for commands
/// * args - Arguments of command line
/// * configurations - Array where the completed configurations are stored
/// * glob_args - Array where global keys are stored (keys that do not belong to specific commands, before '--')
///
/// returns ()
pub fn build_configurations(
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
        "-test-time",
    ];

    let mut current: Configuration = Configuration::new();
    let mut is_glob = true;
    if !args.contains(&"--".to_string()) {
        is_glob = false;
    }
    for i in &args[1..] {
        if is_glob && i == "--" {
            is_glob = false;
        }
        if !is_glob && i == "--" {
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


///
/// Parses one argument of cmd line
/// * container - Array where parts of key are stored
/// * i - Key
/// * keys - Possible keys
/// 
/// returns - bool
/// - true - without errors
/// - false - parse errors (argument will be skipped)
///
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
