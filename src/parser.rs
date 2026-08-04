use crate::lexer::Token;
use std::iter::Peekable;
use std::vec::IntoIter;

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Int,
    Float,
    Str,
    Bool,
    Void,
    Array(Box<Type>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    NoValue,
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Array(Vec<Expr>),
    Ident(String),
    Unary(UnaryOp, Box<Expr>),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    Call(String, Vec<Expr>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOp {
    Not,
    Neg,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    And,
    Or,

    Eq,
    Neq,

    Lt,
    Lte,
    Gt,
    Gte,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Var {
        name: String,
        type_: Type,
        value: Expr,
        immutable: bool,
    },
    Assign {
        name: String,
        value: Expr,
    },
    Fn {
        name: String,
        args: Vec<(bool, Type, String)>,
        returns: Type,
        body: Vec<Node>,
    },
    If {
        condition: Expr,
        then_branch: Vec<Node>,
        else_branch: Option<Vec<Node>>,
    },
    While {
        condition: Expr,
        body: Vec<Node>,
    },
    For {
        var: String,
        iterable: Expr,
        body: Vec<Node>,
    },
    CallFn {
        callee: Expr,
        name: String,
        args: Vec<Expr>,
    },
    Return(Expr),
    Break,
    Continue,
    Try {
        body: Vec<Node>,
        catch: Option<(String, Vec<Node>)>,
    },

    Eot,
}

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: tokens.into_iter().peekable(),
        }
    }

    pub fn parse(&mut self) -> Vec<Node> {
        let mut nodes: Vec<Node> = vec![];
        loop {
            let node: Node = self.next_node();
            if node == Node::Eot {
                break;
            }
            nodes.push(node);
        }
        nodes
    }

    fn next_node(&mut self) -> Node {
        match self.tokens.next() {
            None => Node::Eot,
            Some(tok) => match tok {
                Token::Fn => self.next_function(),
                _ => panic!("Unexpected token: {:?}", tok),
            },
        }
    }

    fn get_type(&mut self) -> (bool, Type) {
        let mut immutable: bool = false;
        let mut type_: Type;

        loop {
            let token: Token = self.get_next_token();
            if token == Token::Immut {
                if immutable {
                    panic!("Multiple 'immut' keywords");
                } else {
                    immutable = true;
                    continue;
                }
            }

            if let Token::Ident(type_name) = token {
                type_ = match type_name.as_ref() {
                    "int" => Type::Int,
                    "float" => Type::Float,
                    "str" => Type::Str,
                    "bool" => Type::Bool,
                    _ => panic!("Invalid type: {}", type_name),
                };

                if self.tokens.peek() == Some(&Token::LeftBracket) {
                    self.tokens.next();
                    if self.get_next_token() != Token::RightBracket {
                        panic!("Expected ']'");
                    }
                    type_ = Type::Array(Box::new(type_));
                }

                break;
            }
        }

        (immutable, type_)
    }

    fn get_value(&mut self, target_type: &Type) -> Expr {
        let value: Expr = match self.get_next_token() {
            Token::NoValue => Expr::NoValue,
            Token::IntLit(value) => {
                if target_type == &Type::Int {
                    Expr::Int(value)
                } else {
                    panic!("Invalid type")
                }
            }
            Token::FloatLit(value) => {
                if target_type == &Type::Float {
                    Expr::Float(value)
                } else {
                    panic!("Invalid type")
                }
            }
            Token::StrLit(value) => {
                if target_type == &Type::Str {
                    Expr::Str(value)
                } else {
                    panic!("Invalid type")
                }
            }
            Token::BoolLit(value) => {
                if target_type == &Type::Bool {
                    Expr::Bool(value)
                } else {
                    panic!("Invalid type")
                }
            }
            Token::LeftBracket => {
                if let Type::Array(inner_type) = target_type {
                    if let Type::Array(_) = **inner_type {
                        panic!("Nested arrays are not supported");
                    }

                    let mut array: Vec<Expr> = vec![];
                    loop {
                        array.push(self.get_value(&inner_type));
                        let token: Token = self.get_next_token();
                        if token == Token::RightBracket {
                            break;
                        }
                        if token != Token::Comma {
                            panic!("Expected ','");
                        }
                    }
                    Expr::Array(array)
                } else {
                    panic!("Invalid type")
                }
            }
            _ => panic!("Expected value"),
        };

        value
    }

    fn get_next_token(&mut self) -> Token {
        match self.tokens.next() {
            Some(tok) => tok,
            None => panic!("Unexpected end of file"),
        }
    }

    fn next_function(&mut self) -> Node {
        let fn_name: String;
        let mut fn_args: Vec<(bool, Type, String)> = vec![];
        let mut fn_body: Vec<Node> = vec![];
        let fn_returns: Type;

        if let Token::Ident(name) = self.get_next_token() {
            fn_name = name
        } else {
            panic!("Expected function name");
        }

        if self.get_next_token() != Token::LeftParen {
            panic!("Expected '('");
        }

        loop {
            let arg_name: String;
            let arg_immutable: bool;
            let arg_type: Type;

            let mut token: Token = self.get_next_token();

            if token == Token::RightParen {
                break;
            }

            if let Token::Ident(name) = token {
                arg_name = name;
            } else {
                panic!("Expected argument name");
            }

            token = self.get_next_token();

            if token != Token::Colon {
                panic!("Expected ':'");
            }

            (arg_immutable, arg_type) = self.get_type();

            fn_args.push((arg_immutable, arg_type, arg_name));

            match self.tokens.next() {
                Some(Token::Comma) => {}
                Some(Token::RightParen) => break,
                _ => panic!("Expected ',' or ')'"),
            }
        }

        {
            if self.get_next_token() != Token::Ret {
                panic!("Expected 'ret'");
            }

            if let Token::Ident(type_) = self.get_next_token() {
                fn_returns = match type_.as_ref() {
                    "int" => Type::Int,
                    "float" => Type::Float,
                    "str" => Type::Str,
                    "bool" => Type::Bool,
                    "void" => Type::Void,
                    _ => panic!("Invalid return type: {}", type_),
                };
            } else {
                panic!("Expected return type");
            }
        }

        self.tokens.next();

        while let Some(tok) = self.tokens.peek() {
            match tok {
                Token::RightBrace => {
                    self.tokens.next();
                    break;
                }
                Token::Var => fn_body.push(self.next_var()),
                _ => panic!("Unexpected token: {:?}", tok),
            }
        }

        Node::Fn {
            name: fn_name,
            args: fn_args,
            returns: fn_returns,
            body: fn_body,
        }
    }

    fn next_var(&mut self) -> Node {
        let var_name: String;

        self.tokens.next();

        if let Token::Ident(name) = self.get_next_token() {
            var_name = name;
        } else {
            panic!("Expected variable name");
        }

        if self.get_next_token() != Token::Colon {
            panic!("Expected ':'");
        }

        let (var_immutable, var_type) = self.get_type();

        if self.get_next_token() != Token::Equal {
            panic!("Expected '='");
        }

        let var_value: Expr = self.get_value(&var_type);

        if self.get_next_token() != Token::Semicolon {
            panic!("Expected ';'");
        }

        Node::Var {
            name: var_name,
            type_: var_type,
            value: var_value,
            immutable: var_immutable,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Token;
    use crate::lexer::Token::Ident;
    use crate::parser::{Expr, Node, Type};

    #[test]
    fn test_fn_parse() {
        let mut parser_1 = crate::parser::Parser::new(vec![
            Ident(String::from("main")),
            Token::LeftParen,
            Token::RightParen,
            Token::Ret,
            Ident(String::from("void")),
            Token::LeftBrace,
            Token::RightBrace,
        ]);
        let fn_node_1 = parser_1.next_function();
        assert_eq!(
            fn_node_1,
            Node::Fn {
                name: String::from("main"),
                args: vec![],
                returns: Type::Void,
                body: vec![]
            }
        );

        let mut parser_2 = crate::parser::Parser::new(vec![
            Ident(String::from("main")),
            Token::LeftParen,
            Ident(String::from("Test")),
            Token::Colon,
            Ident(String::from("int")),
            Token::Comma,
            Ident(String::from("Test2_______")),
            Token::Colon,
            Token::Immut,
            Ident(String::from("str")),
            Token::Comma,
            Ident(String::from("Test3")),
            Token::Colon,
            Ident(String::from("int")),
            Token::LeftBracket,
            Token::RightBracket,
            Token::RightParen,
            Token::Ret,
            Ident(String::from("int")),
            Token::LeftBrace,
            Token::RightBrace,
        ]);
        let fn_node_2 = parser_2.next_function();
        assert_eq!(
            fn_node_2,
            Node::Fn {
                name: String::from("main"),
                args: vec![
                    (false, Type::Int, String::from("Test")),
                    (true, Type::Str, String::from("Test2_______")),
                    (
                        false,
                        Type::Array(Box::new(Type::Int)),
                        String::from("Test3")
                    ),
                ],
                returns: Type::Int,
                body: vec![],
            }
        );
    }

    #[test]
    fn test_var_parse() {
        let mut parser_1 = crate::parser::Parser::new(vec![
            Token::Var,
            Ident(String::from("main")),
            Token::Colon,
            Ident(String::from("int")),
            Token::Equal,
            Token::IntLit(1),
            Token::Semicolon,
            Token::Var,
            Ident(String::from("Test")),
            Token::Colon,
            Token::Immut,
            Ident(String::from("int")),
            Token::Equal,
            Token::NoValue,
            Token::Semicolon,
            Token::Var,
            Ident(String::from("Test2")),
            Token::Colon,
            Ident(String::from("int")),
            Token::LeftBracket,
            Token::RightBracket,
            Token::Equal,
            Token::LeftBracket,
            Token::IntLit(1),
            Token::Comma,
            Token::IntLit(2),
            Token::RightBracket,
            Token::Semicolon,
        ]);
        let var_node_1_1 = parser_1.next_var();
        let var_node_1_2 = parser_1.next_var();
        let var_node_1_3 = parser_1.next_var();
        assert_eq!(
            var_node_1_1,
            Node::Var {
                name: String::from("main"),
                type_: Type::Int,
                value: Expr::Int(1),
                immutable: false,
            }
        );
        assert_eq!(
            var_node_1_2,
            Node::Var {
                name: String::from("Test"),
                type_: Type::Int,
                value: Expr::NoValue,
                immutable: true,
            }
        );
        assert_eq!(
            var_node_1_3,
            Node::Var {
                name: String::from("Test2"),
                type_: Type::Array(Box::new(Type::Int)),
                value: Expr::Array(vec![Expr::Int(1), Expr::Int(2)]),
                immutable: false,
            }
        );
    }
}
