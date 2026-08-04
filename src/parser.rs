use crate::lexer::Token;
use std::iter::Peekable;
use std::vec::IntoIter;

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Int,
    Float,
    Str,
    Bool,
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

    Eot
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
            Some(tok) => {
                match tok {
                    Token::Fn => self.next_function(),
                    _ => panic!("Unexpected token: {:?}", tok),
                }
            }
        }
    }

    fn next_function(&mut self) -> Node {
        let fn_name: String;
        let mut fn_args: Vec<(bool, Type, String)> = vec![];
        let mut fn_body: Vec<Node> = vec![];

        {
            let token: Token = match self.tokens.next() {
                Some(tok) => tok,
                None => panic!("Unexpected end of file"),
            };

            if let Token::Ident(name) = token {
                fn_name = name
            } else {
                panic!("Expected function name");
            }
        }

        {
            let token: Token = match self.tokens.next() {
                Some(tok) => tok,
                None => panic!("Unexpected end of file"),
            };

            if token != Token::LeftParen {
                panic!("Expected '('");
            }

            loop {
                let mut arg_name: String = String::new();
                let mut arg_immutable: bool = false;
                let mut arg_type: Type = Type::Int;

                let mut token: Token = match self.tokens.next() {
                    Some(tok) => tok,
                    None => panic!("Unexpected end of file"),
                };

                if token == Token::RightParen {
                    break;
                }

                if let Token::Ident(name) = token {
                    arg_name = name;
                } else {
                    panic!("Expected argument name");
                }

                token = match self.tokens.next() {
                    Some(tok) => tok,
                    None => panic!("Unexpected end of file"),
                };

                if token != Token::Colon {
                    panic!("Expected ':'");
                }

                token = match self.tokens.next() {
                    Some(tok) => tok,
                    None => panic!("Unexpected end of file"),
                };

                if let Token::Ident(type_) = token {
                    arg_type = match type_.as_ref() {
                        "int" => Type::Int,
                        "float" => Type::Float,
                        "str" => Type::Str,
                        "bool" => Type::Bool,
                        _ => panic!("Invalid type: {}", type_),
                    };
                } else if let Token::Immut = token {
                    arg_immutable = true;

                    token = match self.tokens.next() {
                        Some(tok) => tok,
                        None => panic!("Unexpected end of file"),
                    };

                    if let Token::Ident(type_) = token {
                        arg_type = match type_.as_ref() {
                            "int" => Type::Int,
                            "float" => Type::Float,
                            "str" => Type::Str,
                            "bool" => Type::Bool,
                            _ => panic!("Invalid type: {}", type_),
                        };
                    } else {
                        panic!("Expected type after 'immut'");
                    }
                } else {
                    panic!("Expected type or immut");
                }

                fn_args.push((arg_immutable, arg_type, arg_name));

                match self.tokens.next() {
                    Some(Token::Comma) => {}
                    Some(Token::RightParen) => break,
                    _ => panic!("Expected ',' or ')'"),
                }
            }
        }

        while let Some(tok) = self.tokens.peek() {
            match tok {
                Token::LeftBrace => {},
                Token::RightBrace => break,
                Token::Var => fn_body.push(self.next_var()),
                
                _ => panic!("Unexpected token: {:?}", tok),
            }
        }

        Node::Fn {
            name: fn_name,
            args: fn_args,
            body: fn_body,
        }
    }

    fn next_var(&mut self) -> Node {
        Node::Var {
            name: String::new(),
            type_: Type::Int,
            value: Expr::NoValue,
            immutable: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Token;
    use crate::lexer::Token::Ident;
    use crate::parser::{Node, Type};

    #[test]
    fn test_fn_parse() {
        let mut parser1 = crate::parser::Parser::new(vec![Ident(String::from("main")), Token::LeftParen, Token::RightParen]);
        let fn_node = parser1.next_function();
        assert_eq!(fn_node, Node::Fn { name: String::from("main"), args: vec![], body: vec![] });

        let mut parser2 = crate::parser::Parser::new(vec![Ident(String::from("main")), Token::LeftParen, Ident(String::from("Test")), Token::Colon, Ident(String::from("int")), Token::Comma, Ident(String::from("Test2_______")), Token::Colon, Token::Immut, Ident(String::from("str")), Token::RightParen]);
        let fn_node1 = parser2.next_function();
        assert_eq!(fn_node1, Node::Fn { name: String::from("main"), args: vec![(false, Type::Int, String::from("Test")), (true, Type::Str, String::from("Test2_______"))], body: vec![] });
    }
}
