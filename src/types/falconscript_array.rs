use crate::interpreter::{type_matches, Env, Flow, Value};
use crate::parser::Type;

pub fn eval_array_call(array_name: &String, function_name: &String, args: &Vec<Value>, env: &mut Env) -> Flow {
    match function_name.as_ref() {
        "push" => push(array_name, args, env),
        "rm" => rm(array_name, args, env),
        "at" => at(array_name, args, env),
        "len" => len(array_name, args, env),
        _ => panic!("Invalid array function: {}", function_name),
    }
}

fn push(array_name: &String, args: &Vec<Value>, env: &mut Env) -> Flow {
    let array = env.get(array_name).unwrap_or_else(|| panic!("Undefined variable: {}", array_name));
    if array.1 {
        panic!("Cannot push to immutable array: {}", array_name);
    }
    if args.len() != 1 {
        panic!("Invalid number of arguments for 'array:push'");
    }
    let value: Value = args[0].clone();
    let Type::Array(array_type) = &array.2 else { panic!("Can't run array:push on non-array: {:?}", array.0) };
    if !type_matches(array_type, &value) {
        panic!("Invalid type for array:push {:?}, expected {:?}", value, array_type);
    }

    let Value::Array(mut array_vec) = array.0.clone() else { panic!("Can't run array:push on non-array: {:?}", array.0) };
    array_vec.push(value);
    env.insert(array_name.clone(), (Value::Array(array_vec), array.1, array.2.clone()));

    Flow::Normal
}

fn rm(array_name: &String, args: &Vec<Value>, env: &mut Env) -> Flow {
    let array = env.get(array_name).unwrap_or_else(|| panic!("Undefined variable: {}", array_name));
    if array.1 {
        panic!("Cannot remove from immutable array: {}", array_name);
    }
    if args.len() != 1 {
        panic!("Invalid number of arguments for 'array:rm'");
    }
    let Value::Int(index) = args[0].clone() else { panic!("Invalid argument for 'array:rm': {:?}", args[0]) };

    let Value::Array(mut array_vec) = array.0.clone() else { panic!("Can't run array:rm on non-array: {:?}", array.0) };
    array_vec.remove(index as usize);
    env.insert(array_name.clone(), (Value::Array(array_vec), array.1, array.2.clone()));

    Flow::Normal
}

fn at(array_name: &String, args: &Vec<Value>, env: &mut Env) -> Flow {
    let Value::Array(array_vec) = env.get(array_name).unwrap_or_else(|| panic!("Undefined variable: {}", array_name)).0.clone() else { panic!("Can't run array:at on non-array: {:?}", array_name) };
    if args.len() != 1 {
        panic!("Invalid number of arguments for 'array:at'");
    }
    let Value::Int(index) = args[0].clone() else { panic!("Invalid argument for 'array:at': {:?}", args[0]) };
    if index < 0 || index >= array_vec.len() as i64 {
        panic!("Index out of bounds: {}", index);
    }
    Flow::Return(array_vec[index as usize].clone())
}

fn len(array_name: &String, args: &Vec<Value>, env: &mut Env) -> Flow {
    let Value::Array(array_vec) = env.get(array_name).unwrap_or_else(|| panic!("Undefined variable: {}", array_name)).0.clone() else { panic!("Can't run array:len on non-array: {:?}", array_name) };
    if args.len() != 0 {
        panic!("Invalid number of arguments for 'array:len'");
    }
    Flow::Return(Value::Int(array_vec.len() as i64))
}
