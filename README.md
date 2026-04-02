# RT - Time Executor
This program allows you to measure the execution time 
of commands and other programs. 

1. [Building](#building)
2. [Usage](#usage-examples)
3. [Keys](#keys)

## 1. Building

### 1. Building program 
```shell
cargo build --release
```
### 2. Usage
```shell
rt -c="echo \"Hello\""
```

## 2. Usage examples

### 1. Print help message
```shell
rt
```
### 2. Measure execution time
```shell
rt -c="echo <some text>"
```
### 3. Usage with global keys
```shell
# -time ans -proc will be used for all commands
rt -time="s" -proc="bash" -- -c="echo 1" -c="mkdir directory" 
```

### 4. Global and local keys
```shell
# global -time will be used only for first -c (echo 1). 
# For -c="touch file.txt" will be used it local key -time
rt -time="s" -- -c="echo 1" -time="ms" -c="touch file.txt "
```

## 3. Keys
The following options can be used during testing:
- __-min__ - suppresses the standard output and error streams;
- __-err__ - suppresses the standard output but displays the error stream;
- __-time=\<time>__ - sets the time unit for the command. Used to display execution time and for comparison. The following values are possible: ms (milliseconds, by default), s (seconds), m (minutes), h (hours)
- __-proc=\<interpreter>__ - sets the command interpreter for the program. The following values are possible: bash, sh (default for Linux), zsh, cmd (default for Windows), powershell
- __-c=“command”__ - the program itself, whose execution time will be measured (can be written together with arguments
- __--__ - The boundary between global options (options that apply to all commands) and local ones. Optional if there are no global keys
