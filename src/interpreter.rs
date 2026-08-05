use crate::parser::{Expr, Node, Type};
use std::collections::HashMap;
use std::fmt;
use std::io::Write;

#[derive(Debug, PartialEq, Clone)]
enum Value {
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

fn type_matches(type_: &Type, value: &Value) -> bool {
    matches!(
        (type_, value),
        (Type::Int, Value::Int(_))
            | (Type::Float, Value::Float(_))
            | (Type::Str, Value::Str(_))
            | (Type::Bool, Value::Bool(_))
            | (Type::Array(_), Value::Array(_))
    )
}

struct FuncInfo {
    args: Vec<(bool, Type, String)>,
    returns: Type,
    body: Vec<Node>,
}

type Env = HashMap<String, (Value, bool)>;

enum Flow {
    Normal,
    Return(Value),
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
            if let Node::Fn { name, args, returns, body } = node {
                self.functions.insert(
                    name.clone(),
                    FuncInfo {
                        args: args.clone(),
                        returns: returns.clone(),
                        body: body.clone(),
                    }
                );
            } else {
                panic!("Unexpected node {:?} in the top level!", node);
            }
        }

        if let Some(main) = self.functions.get("main") {
            if main.args.len() != 1 {
                panic!("'main' function must take exactly one argument!");
            }
            if main.args[0].1 != Type::Array(Box::new(Type::Str)) || main.args[0].0 != true || main.args[0].2 != "args" {
                panic!("'main' function must take 'args' as its only argument!");
            }
            if main.returns != Type::Int {
                panic!("'main' function must return 'int'!");
            }

            let mut env: Env = HashMap::new();
            let mut program_args: Vec<Value> = vec![];
            for arg in args {
                program_args.push(Value::Str(arg.clone()))
            }
            env.insert("args".to_string(), (Value::Array(program_args), true));

            for node in &main.body {
                if let Flow::Return(ret_val) = self.exec_node(node) {
                    if matches!(ret_val, Value::Int(_)) {
                        println!("{}", ret_val);
                    } else {
                        panic!("'main' function must return 'int'!");
                    }
                }
            }
        }
    }

    fn exec_node(&mut self, node: &Node) -> Flow {

    }
}
