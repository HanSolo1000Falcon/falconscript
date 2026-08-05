use std::io::Read;
use crate::interpreter::{Flow, Value};

pub fn eval_std_call(function_name: &String, args: &Vec<Value>) -> Flow {
    match function_name.as_ref() {
        "random" => random(),
        "read_from_file" => read_from_file(&args),
        "format" => format(&args),
        _ => panic!("The function {} is not a part of the falconscript std lib", function_name),
    }
}

fn random() -> Flow {
    Flow::Return(Value::Float(rand::random::<f64>()))
}

fn read_from_file(args: &Vec<Value>) -> Flow {
    if args.len() != 1 {
        panic!("std:read_from_file takes exactly one argument!");
    }

    let Value::Str(file_path) = &args[0] else { panic!("std:read_from_file takes a string as its argument!") };
    let mut file = std::fs::File::open(file_path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    Flow::Return(Value::Str(contents))
}

fn format(args: &Vec<Value>) -> Flow {
    if args.len() < 1 {
        panic!("std:format takes at least one argument!");
    }

    let Value::Str(format_string) = &args[0] else { panic!("std:format takes a string as its first argument!") };
    let mut format_string = format_string.clone();
    for arg in args.iter().skip(1) {
        format_string = format_string.replacen("{}", &arg.to_string(), 1);
    }
    Flow::Return(Value::Str(format_string.clone()))
}