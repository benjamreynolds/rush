#![allow(unused_imports)] //Here until interpret is complete
use std::process::*;
use process::logic::*;
use process::stdproc::*;
use process::pipe::*;
use process::ops::*;
use process::pq::*;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

///Interpret
///Given an input command, interpret parses and determines what and how
///to execute it and returns output or error output
pub fn interpret(command: String) -> bool {
    let mut op_queues = Opqueue::new();
    let mut proc_queue = Procqueue::new();
    let command: Vec<&str> = command.trim().split(' ').collect();

    //Split order:
    //Split by parallel +=+
    //Split by or ||
    //Split by pipe |
    //Split by and &&
    //Split by (To be expanded)

    let mut redirects = false;
    let mut pipes = false;
    for i in command.clone() {
        if i.contains('>') {
            redirects = true;
        }
        if i.contains('|') && !i.contains("||") {
            pipes = true;
        }
    }
    if pipes {
        //Pipe or no pipe
        piped(command)
    } else if redirects {
        redirect(command)
    } else {
        //execute normally
        run(command)
    }
}

///Run
///Runs commands passed to it and returns the output
pub fn run(command: Vec<&str>) -> bool {
    let args = command.as_slice();
    if args.len() > 1 {
        let mut cmd = Command::new(&args[0])
            .args(&args[1..])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Command failed to start");
        let status = cmd.wait().expect("failed to wait for child");
        ;
        status.success()
    } else if args.len() == 1 {
        let mut cmd = Command::new(&args[0])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Command failed to start");
        let status = cmd.wait().expect("failed to wait for child");
        status.success()
    } else {
        let mut cmd = Command::new("")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Command failed to start");
        let status = cmd.wait().expect("failed to wait for child");
        status.success()
    }
}

pub fn redirect(command: Vec<&str>) -> bool {
    let mut args = command;
    let mut file_path = "".to_owned();
    for i in 0..args.len() {
        if args[i].contains('>') {
            file_path.push_str(&args[i + 1..args.len()].to_vec().join(""));
            args.truncate(i);
            break;
        }
    }
    let args = args.as_slice();
    let output = if args.len() > 1 {
        Command::new(&args[0]).args(&args[1..]).output().ok()
    } else if args.len() == 1 {
        Command::new(&args[0]).output().ok()
    } else {
        Command::new("").output().ok()
    };
    let str_out = if output.is_some() {
        let temp = output.expect("Output has been checked");
        if temp.stdout.is_empty() {
            String::from_utf8(temp.stderr)
                .expect("Should have translated to string easily")
        } else {
            String::from_utf8(temp.stdout)
                .expect("Should have translated to string easily")
        }
    } else {
        "".to_owned()
    };
    let path = Path::new(&file_path);
    let display = path.display();
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why.description()),
        Ok(file) => file,
    };
    if let Err(why) = file.write_all(str_out.as_bytes()) {
        panic!("couldn't write to {}: {}", display, why.description());
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execute_test() {
        let vec = "ls -al".to_owned();
        let result = interpret(vec);
        assert!(!result.is_empty());
    }

    #[test]
    fn execute_fail() {
        let vec = "blah".to_owned();
        let result = interpret(vec);
        assert_eq!("Please input a valid command",result);
    }
}

