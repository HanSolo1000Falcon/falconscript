use crate::falconscript_stdlib::eval_std_call;
use crate::parser::{BinaryOp, Expr, Node, Type, UnaryOp};
use crate::types::falconscript_array::eval_array_call;
use std::collections::HashMap;
use std::io::Write;
use std::{fmt, io};
use crate::types::falconscript_str::eval_str_call;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Array(Vec<Value>),
    NoValue,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{n}"),
            Value::Float(x) => write!(f, "{x}"),
            Value::Str(s) => write!(f, "{s}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Array(a) => write!(f, "{:?}", a),
            Value::NoValue => write!(f, "none"),
        }
    }
}

pub fn type_matches(type_: &Type, value: &Value) -> bool {
    matches!(
        (type_, value),
        (Type::Int, Value::Int(_))
            | (Type::Float, Value::Float(_))
            | (Type::Str, Value::Str(_))
            | (Type::Bool, Value::Bool(_))
            | (Type::Array(_), Value::Array(_))
    ) || matches!(value, Value::NoValue)
}

struct FuncInfo {
    args: Vec<(bool, Type, String)>,
    returns: Type,
    body: Vec<Node>,
}

pub type Env = HashMap<String, (Value, bool, Type)>;

pub enum Flow {
    Normal,
    Return(Value),
    Break,
    Continue,
}

pub struct Interpreter<W: Write> {
    code: Vec<Node>,
    functions: HashMap<String, FuncInfo>,
    stdout: W,
}

impl<W: Write> Interpreter<W> {
    pub fn new(code: Vec<Node>, out: W) -> Interpreter<W> {
        Interpreter {
            code,
            functions: HashMap::new(),
            stdout: out,
        }
    }

    pub fn run(&mut self, args: &Vec<String>) {
        for node in &self.code {
            if let Node::Fn {
                name,
                args,
                returns,
                body,
            } = node
            {
                self.functions.insert(
                    name.clone(),
                    FuncInfo {
                        args: args.clone(),
                        returns: returns.clone(),
                        body: body.clone(),
                    },
                );
            } else {
                panic!("Unexpected node {:?} in the top level!", node);
            }
        }

        let (main_body, main_args, main_returns) = if let Some(main) = self.functions.get("main") {
            (main.body.clone(), main.args.clone(), main.returns.clone())
        } else {
            return;
        };

        if main_args.len() != 1 {
            panic!("'main' function must take exactly one argument!");
        }
        if main_args[0].1 != Type::Array(Box::new(Type::Str))
            || main_args[0].0 != true
            || main_args[0].2 != "args"
        {
            panic!("'main' function must take 'args' as its only argument!");
        }
        if main_returns != Type::Int {
            panic!("'main' function must return 'int'!");
        }

        let mut env: Env = HashMap::new();
        let mut program_args: Vec<Value> = vec![];
        for arg in args {
            program_args.push(Value::Str(arg.clone()))
        }
        env.insert(
            "args".to_string(),
            (
                Value::Array(program_args),
                true,
                Type::Array(Box::new(Type::Str)),
            ),
        );

        if let Flow::Return(ret_val) = self.exec_block(&main_body, &mut env) {
            if let Value::Int(ret_val) = ret_val {
                std::process::exit(ret_val as i32);
            } else {
                panic!("'main' function must return 'int'!");
            }
        }
    }

    fn exec_block(&mut self, block: &Vec<Node>, env: &mut Env) -> Flow {
        for node in block {
            match self.exec_node(&node, env) {
                Flow::Normal => {}
                flow => return flow,
            }
        }
        Flow::Normal
    }

    fn exec_node(&mut self, node: &Node, env: &mut Env) -> Flow {
        match node {
            Node::Var {
                name,
                type_,
                value,
                immutable,
            } => {
                let value_as_val: Value = self.eval(value, env);
                if !type_matches(&type_, &value_as_val) {
                    panic!("Invalid type for variable: {:?} = {:?}", name, value);
                }
                env.insert(name.clone(), (value_as_val, *immutable, type_.clone()));
                Flow::Normal
            }
            Node::Assign { name, value } => {
                if !env.contains_key(name) {
                    panic!("Undefined variable: {}", name);
                }
                if env.get(name).unwrap().1 {
                    panic!("Cannot assign to immutable variable: {}", name);
                }
                let value_as_val: Value = self.eval(value, env);
                if !type_matches(&env.get(name).unwrap().2, &value_as_val) {
                    panic!("Invalid type for variable: {:?} = {:?}", name, value);
                }
                env.insert(
                    name.clone(),
                    (
                        value_as_val,
                        env.get(name).unwrap().1,
                        env.get(name).unwrap().2.clone(),
                    ),
                );
                Flow::Normal
            }
            Node::CallFn { callee, name, args } => {
                let args_as_vals: Vec<Value> = args.iter().map(|a| self.eval(a, env)).collect();
                if let Expr::Ident(callee_name) = callee {
                    if callee_name == "std" {
                        return match name.clone().as_ref() {
                            "println" | "print" => {
                                if args_as_vals.len() < 1 {
                                    panic!("Invalid number of arguments for 'std:print(ln)'");
                                }

                                let Value::Str(string) = &args_as_vals[0] else {
                                    panic!("Invalid argument for 'std:print(ln)'")
                                };
                                let mut to_print = string.clone();
                                for i in 1..args_as_vals.len() {
                                    to_print =
                                        to_print.replacen("{}", &args_as_vals[i].to_string(), 1);
                                }
                                if name == "println" {
                                    to_print += "\n";
                                }
                                self.stdout.write_all(to_print.as_bytes()).unwrap();
                                Flow::Normal
                            }
                            "getln" => {
                                let mut input = String::new();
                                io::stdin().read_line(&mut input).unwrap();
                                Flow::Return(Value::Str(input.trim().to_string()))
                            }
                            "exit" => {
                                let exit_code: i32 = if args.len() == 0 {
                                    0
                                } else {
                                    let Value::Int(code) = args_as_vals[0] else {
                                        panic!("Invalid argument for 'std:exit'")
                                    };
                                    code as i32
                                };
                                if args.len() > 2 {
                                    panic!("Invalid number of arguments for 'std:exit'");
                                } else if args.len() == 2 {
                                    let Value::Str(message) = args_as_vals[1].clone() else {
                                        panic!("Invalid argument for 'std:exit'")
                                    };
                                    self.stdout.write_all(message.as_bytes()).unwrap();
                                }
                                std::process::exit(exit_code);
                            }
                            _ => eval_std_call(name, &args_as_vals),
                        };
                    }

                    if env.contains_key(callee_name) {
                        let callee_info = env.get(callee_name).unwrap();
                        return match callee_info.2 {
                            Type::Array(_) => eval_array_call(callee_name, name, &args_as_vals, env),
                            Type::Str => eval_str_call(callee_name, name, &args_as_vals, env),
                            _ => panic!("Invalid function call: {}:{}", callee_name, name),
                        };
                    }

                    panic!(
                        "Attempted to call a function on undefined callee: {}",
                        callee_name
                    );
                }

                if self.functions.contains_key(name) {
                    let (func_body, func_args, func_return) =
                        if let Some(func) = self.functions.get(name) {
                            (func.body.clone(), func.args.clone(), func.returns.clone())
                        } else {
                            unreachable!()
                        };
                    let mut func_env: Env = HashMap::new();
                    if args_as_vals.len() != func_args.len() {
                        panic!("Invalid number of arguments for function: {}", name);
                    }

                    for i in 0..func_args.len() {
                        let (arg_immutable, arg_type, arg_name) = func_args[i].clone();
                        let arg_value = args_as_vals[i].clone();
                        if !type_matches(&arg_type, &arg_value) {
                            panic!("Invalid type for argument {} of function: {:?}", i, name);
                        }
                        func_env.insert(arg_name, (arg_value, arg_immutable, arg_type));
                    }

                    return self.exec_block(&func_body, &mut func_env);
                }

                panic!("Attempted to call an undefined function: {}", name);
            }
            Node::If { condition, then_branch, else_branch } => {
                if self.eval_bool(condition, env) {
                    let mut then_env = env.clone();
                    let flow: Flow = self.exec_block(then_branch, &mut then_env);
                    for (key, value) in then_env.iter() {
                        if env.contains_key(key) {
                            env.insert(key.clone(), (value.0.clone(), value.1, value.2.clone()));
                        }
                    }
                    flow
                } else {
                    if let Some(else_branch) = else_branch {
                        let mut else_env = env.clone();
                        let flow: Flow = self.exec_block(else_branch, &mut else_env);
                        for (key, value) in else_env.iter() {
                            if env.contains_key(key) {
                                env.insert(key.clone(), (value.0.clone(), value.1, value.2.clone()));
                            }
                        }
                        flow
                    } else {
                        Flow::Normal
                    }
                }
            }
            Node::While { condition, body } => {
                let mut while_env = env.clone();
                while self.eval_bool(condition, env) {
                    let flow: Flow = self.exec_block(body, &mut while_env);

                    for (key, value) in while_env.iter() {
                        if env.contains_key(key) {
                            env.insert(key.clone(), (value.0.clone(), value.1, value.2.clone()));
                        }
                    }

                    if let Flow::Return(_) = flow {
                        return flow;
                    } else if matches!(flow, Flow::Break) {
                        break;
                    }
                }
                Flow::Normal
            }
            Node::For { var, iterable, body } => {
                let mut for_env = env.clone();
                match iterable {
                    Expr::Range(from, to) => {
                        if let Node::Var { name, type_, value, immutable } = var.as_ref() {
                            if type_ != &Type::Int {
                                panic!("Invalid type for variable: {:?}", name);
                            }

                            let mut i: i64 = from.clone();
                            while i < *to {
                                for_env.insert(name.clone(), (Value::Int(i), *immutable, Type::Int));
                                let flow: Flow = self.exec_block(body, &mut for_env.clone());

                                for (key, value) in for_env.iter() {
                                    if env.contains_key(key) {
                                        env.insert(key.clone(), (value.0.clone(), value.1, value.2.clone()));
                                    }
                                }

                                if let Flow::Return(_) = flow {
                                    return flow;
                                } else if matches!(flow, Flow::Break) {
                                    break;
                                }
                                i += 1;
                            }
                            return Flow::Normal;
                        }
                        panic!("Invalid iterable expression: {:?}", iterable);
                    }
                    Expr::Ident(iter_name) => {
                        if let Node::Var { name, type_, value, immutable } = var.as_ref() {
                            if let Some(value) = env.clone().get(iter_name) {
                                if let Value::Array(arr) = &value.0 {
                                    let Type::Array(arr_type) = &value.2 else { panic!("Can't iteratet over the type {:?}", &value.2)};
                                    if arr_type.as_ref() != type_ {
                                        panic!("Invalid type for variable: {:?}", name);
                                    }
                                    for i in 0..arr.len() {
                                        for_env.insert(name.clone(), (arr[i].clone(), *immutable, type_.clone()));
                                        let flow: Flow = self.exec_block(body, &mut for_env.clone());
                                        for (key, value) in for_env.iter() {
                                            if env.contains_key(key) {
                                                env.insert(key.clone(), (value.0.clone(), value.1, value.2.clone()));
                                            }
                                        }
                                        if let Flow::Return(_) = flow {
                                            return flow;
                                        } else if matches!(flow, Flow::Break) {
                                            break;
                                        }
                                    }
                                    return Flow::Normal;
                                }
                            }
                            panic!("Invalid iterable expression: {:?}", iterable);
                        }
                        panic!("Invalid iterable expression: {:?}", iterable);
                    }
                    _ => panic!("Invalid iterable expression: {:?}", iterable),
                }
            }
            Node::Return(expr) => Flow::Return(self.eval(expr, env)),
            Node::Break => Flow::Break,
            Node::Continue => Flow::Continue,
            _ => Flow::Normal,
        }
    }

    fn eval_bool(&mut self, expr: &Expr, env: &mut Env) -> bool {
        match self.eval(expr, env) {
            Value::Bool(b) => b,
            _ => panic!("Invalid boolean expression: {:?}", expr),
        }
    }

    fn eval(&mut self, expr: &Expr, env: &mut Env) -> Value {
        match expr {
            Expr::NoValue => Value::NoValue,
            Expr::Int(n) => Value::Int(*n),
            Expr::Float(x) => Value::Float(*x),
            Expr::Str(s) => Value::Str(s.clone()),
            Expr::Bool(b) => Value::Bool(*b),
            Expr::Array(arr) => Value::Array(arr.iter().map(|e| self.eval(e, env)).collect()),
            Expr::Ident(name) => env
                .get(name)
                .unwrap_or_else(|| panic!("Undefined variable: {}", name))
                .0
                .clone(),
            Expr::Unary(op, inner) => {
                let v: Value = self.eval(inner, env);
                match (op, v.clone()) {
                    (UnaryOp::Not, Value::Bool(b)) => Value::Bool(!b),
                    _ => panic!("Invalid unary operation: {:?} on {:?}", op, v),
                }
            }
            Expr::Binary(left, op, right) => {
                let lv: Value = self.eval(left, env);
                let rv: Value = self.eval(right, env);
                eval_binary_op(op.clone(), lv, rv)
            }
            Expr::Node(node) => {
                if let Flow::Return(ret_val) = self.exec_node(node, env) {
                    ret_val
                } else {
                    panic!("Failed to eval node {:?}", node);
                }
            }
            _ => panic!("Invalid expression: {:?}", expr),
        }
    }
}

fn as_f64(v: &Value) -> f64 {
    match v {
        Value::Int(i) => *i as f64,
        Value::Float(f) => *f,
        _ => panic!("Cannot convert {:?} to f64", v),
    }
}

fn eval_binary_op(op: BinaryOp, left: Value, right: Value) -> Value {
    use BinaryOp::*;
    match op {
        And | Or => {
            let (Value::Bool(l), Value::Bool(r)) = (&left, &right) else {
                panic!("Invalid binary operation: {:?} on {:?}", left, right);
            };
            Value::Bool(if op == And { *l && *r } else { *l || *r })
        }
        Eq | Neq => Value::Bool(if op == Eq {
            left == right
        } else {
            left != right
        }),
        Lt | Gt => {
            let (l, r) = (as_f64(&left), as_f64(&right));
            Value::Bool(if op == Lt { l < r } else { l > r })
        }
        Lte | Gte => {
            let (l, r) = (as_f64(&left), as_f64(&right));
            Value::Bool(if op == Lte { l <= r } else { l >= r })
        }
        Add if matches!((&left, &right), (Value::Str(_), Value::Str(_))) => {
            let (Value::Str(l), Value::Str(r)) = (&left, &right) else {
                unreachable!()
            };
            Value::Str(l.clone() + &r)
        }
        Add | Sub | Mul | Div | Mod => {
            if let (Value::Int(l), Value::Int(r)) = (&left, &right) {
                let (l, r) = (*l, *r);
                return Value::Int(match op {
                    Add => l + r,
                    Sub => l - r,
                    Mul => l * r,
                    Div => l / r,
                    Mod => l % r,
                    _ => unreachable!(),
                });
            }

            let (l, r) = (as_f64(&left), as_f64(&right));
            Value::Float(match op {
                Add => l + r,
                Sub => l - r,
                Mul => l * r,
                Div => l / r,
                Mod => l % r,
                _ => unreachable!(),
            })
        }
    }
}
