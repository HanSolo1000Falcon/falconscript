use crate::lexer::Token;

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
    Node(Box<Node>),
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
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos).cloned()
    }

    fn peek_at(&self, offset: usize) -> Option<Token> {
        self.tokens.get(self.pos + offset).cloned()
    }

    fn advance(&mut self) -> Option<Token> {
        let tok: Option<Token> = self.tokens.get(self.pos).cloned();
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    pub fn parse(&mut self) -> Vec<Node> {
        let mut nodes: Vec<Node> = vec![];
        loop {
            let node: Node = self.next_node();
            if node == Node::Eot {
                break;
            }
            nodes.push(node);
            println!("{:#?}", nodes);
        }
        nodes
    }

    fn next_node(&mut self) -> Node {
        match self.advance() {
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
            let token: Token = self.advance().unwrap();
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

                if self.peek().unwrap() == Token::LeftBracket {
                    self.advance();
                    if self.advance().unwrap() != Token::RightBracket {
                        panic!("Expected ']'");
                    }
                    type_ = Type::Array(Box::new(type_));
                }

                break;
            }
        }

        (immutable, type_)
    }

    fn is_binary_op(&self, token: &Token) -> bool {
        matches!(token, Token::Plus | Token::Minus | Token::Star | Token::Slash | Token::Percent)
    }

    fn is_unary_op(&self, token: &Token) -> bool {
        matches!(token, Token::Not | Token::Minus)
    }

    fn to_binary_op(&self, token: &Token) -> Option<BinaryOp> {
        match token {
            Token::Plus => Some(BinaryOp::Add),
            Token::Minus => Some(BinaryOp::Sub),
            Token::Star => Some(BinaryOp::Mul),
            Token::Slash => Some(BinaryOp::Div),
            Token::Percent => Some(BinaryOp::Mod),
            _ => None,
        }
    }

    fn to_unary_op(&self, token: &Token) -> Option<UnaryOp> {
        match token {
            Token::Not => Some(UnaryOp::Not),
            Token::Minus => Some(UnaryOp::Neg),
            _ => None,
        }
    }

    fn get_value(&mut self, target_type: &Type) -> Expr {
        match self.advance().unwrap() {
            Token::NoValue => Expr::NoValue,
            Token::Ident(ident) => Expr::Ident(ident),
            Token::IntLit(value) if matches!(target_type, &Type::Int | &Type::Void) => {
                Expr::Int(value)
            }
            Token::FloatLit(value) if matches!(target_type, &Type::Float | &Type::Void) => {
                Expr::Float(value)
            }
            Token::StrLit(value) if matches!(target_type, &Type::Str | &Type::Void) => {
                Expr::Str(value)
            }
            Token::BoolLit(value) if matches!(target_type, &Type::Bool | &Type::Void) => {
                Expr::Bool(value)
            }
            Token::LeftBracket => {
                if let Type::Array(inner_type) = target_type {
                    if let Type::Array(_) = **inner_type {
                        panic!("Nested arrays are not supported");
                    }

                    let mut array: Vec<Expr> = vec![];
                    loop {
                        array.push(self.get_value(&inner_type));
                        let token: Token = self.advance().unwrap();
                        if token == Token::RightBracket {
                            break;
                        }
                        if token != Token::Comma {
                            panic!("Expected ','");
                        }
                    }
                    Expr::Array(array)
                } else if target_type == &Type::Void {
                    let mut array: Vec<Expr> = vec![];
                    loop {
                        array.push(self.get_value(&Type::Void));
                        let token: Token = self.advance().unwrap();
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
            _ => panic!("Expected value of correct type"),
        }
    }

    fn clean_semicolon(&mut self) {
        if self.advance().unwrap() != Token::Semicolon {
            panic!("Expected ';'");
        }
    }

    fn next_function(&mut self) -> Node {
        let fn_name: String;
        let mut fn_args: Vec<(bool, Type, String)> = vec![];
        let mut fn_body: Vec<Node> = vec![];
        let fn_returns: Type;

        if let Token::Ident(name) = self.advance().unwrap() {
            fn_name = name
        } else {
            panic!("Expected function name");
        }

        if self.advance().unwrap() != Token::LeftParen {
            panic!("Expected '('");
        }

        loop {
            let arg_name: String;
            let arg_immutable: bool;
            let arg_type: Type;

            let mut token: Token = self.advance().unwrap();

            if token == Token::RightParen {
                break;
            }

            if let Token::Ident(name) = token {
                arg_name = name;
            } else {
                panic!("Expected argument name");
            }

            token = self.advance().unwrap();

            if token != Token::Colon {
                panic!("Expected ':'");
            }

            (arg_immutable, arg_type) = self.get_type();

            fn_args.push((arg_immutable, arg_type, arg_name));

            match self.advance() {
                Some(Token::Comma) => {}
                Some(Token::RightParen) => break,
                _ => panic!("Expected ',' or ')'"),
            }
        }

        {
            if self.advance().unwrap() != Token::Ret {
                panic!("Expected 'ret'");
            }

            if let Token::Ident(type_) = self.advance().unwrap() {
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

        self.advance();

        while let Some(tok) = self.peek() {
            println!("{:#?}", fn_body);
            match tok {
                Token::RightBrace => {
                    self.advance();
                    break;
                }
                Token::Var => fn_body.push(self.next_var()),
                Token::If => fn_body.push(self.next_if()),
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

        self.advance();

        if let Token::Ident(name) = self.advance().unwrap() {
            var_name = name;
        } else {
            panic!("Expected variable name");
        }

        if self.advance().unwrap() != Token::Colon {
            panic!("Expected ':'");
        }

        let (var_immutable, var_type) = self.get_type();

        if self.advance().unwrap() != Token::Equal {
            panic!("Expected '='");
        }

        let var_value: Expr = self.get_value(&var_type);

        self.clean_semicolon();

        Node::Var {
            name: var_name,
            type_: var_type,
            value: var_value,
            immutable: var_immutable,
        }
    }

    fn next_if(&mut self) -> Node {
        self.advance();
        let condition: Vec<Node> = self.get_condition();
    }

    fn get_condition(&mut self) -> Vec<Node> {

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
