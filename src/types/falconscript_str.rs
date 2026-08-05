use crate::interpreter::{Env, Flow, Value};

pub fn eval_str_call(str_name: &String, function_name: &String, args: &Vec<Value>, env: &mut Env) -> Flow {
    match function_name.as_ref() {
        "at" => at(str_name, args, env),
        "len" => len(str_name, args, env),
        _ => panic!("Invalid string function: {}", function_name),
    }
}

fn at(str_name: &String, args: &Vec<Value>, env: &Env) -> Flow {
    let str_var = env.get(str_name).unwrap_or_else(|| panic!("{} is not defined in the current scope", str_name));
    let Value::Str(str_value) = str_var.0.clone() else { panic!("{} is not a string", str_name) };
    if args.len() != 1 {
        panic!("Invalid number of arguments for 'str:at'");
    }

    let Value::Int(idx) = args[0] else { panic!("Expected an integer for str:at") };

    if idx < 0 || idx >= str_value.len() as i64 {
        panic!("Index out of bounds: {}", idx);
    }

    Flow::Return(Value::Str(str_value.chars().nth(idx as usize).unwrap().to_string()))
}

fn len(str_name: &String, args: &Vec<Value>, env: &Env) -> Flow {
    let str_var = env.get(str_name).unwrap_or_else(|| panic!("{} is not defined in the current scope", str_name));
    let Value::Str(str_value) = str_var.0.clone() else { panic!("{} is not a string", str_name) };
    if args.len() != 0 {
        panic!("Invalid number of arguments for 'str:len'");
    }
    Flow::Return(Value::Int(str_value.len() as i64))
}