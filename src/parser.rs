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
    Range(i64, i64),
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
        var: Box<Node>,
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

    fn get_value_with_conditions_and_ops(&mut self) -> Expr {
        self.parse_or_condition()
    }

    fn get_value(&mut self) -> Expr {
        match self.advance().unwrap() {
            Token::NoValue => Expr::NoValue,
            Token::Ident(ident) => match self.peek().unwrap() {
                Token::Colon | Token::LeftParen => {
                    self.pos -= 1;
                    Expr::Node(Box::new(self.next_call_fn()))
                }
                _ => Expr::Ident(ident),
            },
            Token::IntLit(value) => Expr::Int(value),
            Token::FloatLit(value) => Expr::Float(value),
            Token::Minus => match self.advance().unwrap() {
                Token::FloatLit(value) => Expr::Float(-value),
                Token::IntLit(value) => Expr::Int(-value),
                _ => panic!("Expected number"),
            },
            Token::StrLit(value) => Expr::Str(value),
            Token::BoolLit(value) => Expr::Bool(value),
            Token::LeftBracket => {
                let mut array: Vec<Expr> = vec![];
                loop {
                    array.push(self.get_value());
                    let token: Token = self.advance().unwrap();
                    if token == Token::RightBracket {
                        break;
                    }
                    if token != Token::Comma {
                        panic!("Expected ','");
                    }
                }
                Expr::Array(array)
            }
            token => panic!("Expected value but got {:?}", token),
        }
    }

    fn parse_minus_op(&mut self) -> Expr {
        let mut left: Expr = self.parse_plus_op();
        while self.peek() == Some(Token::Minus) {
            self.advance();
            let right: Expr = self.parse_plus_op();
            left = Expr::Binary(Box::new(left), BinaryOp::Sub, Box::new(right));
        }
        left
    }

    fn parse_plus_op(&mut self) -> Expr {
        let mut left: Expr = self.parse_div_op();
        while self.peek() == Some(Token::Plus) {
            self.advance();
            let right: Expr = self.parse_div_op();
            left = Expr::Binary(Box::new(left), BinaryOp::Add, Box::new(right));
        }
        left
    }

    fn parse_div_op(&mut self) -> Expr {
        let mut left: Expr = self.parse_mod_op();
        while self.peek() == Some(Token::Slash) {
            self.advance();
            let right: Expr = self.parse_mod_op();
            left = Expr::Binary(Box::new(left), BinaryOp::Div, Box::new(right));
        }
        left
    }

    fn parse_mod_op(&mut self) -> Expr {
        let mut left: Expr = self.parse_mul_op();
        while self.peek() == Some(Token::Percent) {
            self.advance();
            let right: Expr = self.parse_mul_op();
            left = Expr::Binary(Box::new(left), BinaryOp::Mod, Box::new(right));
        }
        left
    }

    fn parse_mul_op(&mut self) -> Expr {
        let mut left: Expr = self.parse_primary_op();
        while self.peek() == Some(Token::Star) {
            self.advance();
            let right: Expr = self.parse_primary_op();
            left = Expr::Binary(Box::new(left), BinaryOp::Mul, Box::new(right));
        }
        left
    }

    fn parse_primary_op(&mut self) -> Expr {
        match self.advance() {
            Some(Token::LeftParen) => {
                let expr: Expr = self.parse_minus_op();
                if self.advance() != Some(Token::RightParen) {
                    panic!("Expected ')'");
                }
                expr
            }
            Some(_) => {
                self.pos -= 1;
                self.get_value()
            }
            None => panic!("Unexpected end of input"),
        }
    }

    fn clean_semicolon(&mut self) {
        match self.advance().unwrap() {
            Token::Semicolon => {}
            token => panic!("Expected ';' but got {:?}", token),
        }
    }

    fn get_next(&mut self) -> Node {
        match self.peek().unwrap() {
            Token::Var => self.next_var(),
            Token::If => self.next_if(),
            Token::While => self.next_while(),
            Token::For => self.next_for(),
            Token::Ret => self.next_return(),
            Token::Break => {
                self.advance();
                self.clean_semicolon();
                Node::Break
            }
            Token::Continue => {
                self.advance();
                self.clean_semicolon();
                Node::Continue
            }
            Token::Ident(_) => {
                let to_return: Node = match self.peek_at(1).unwrap() {
                    Token::Colon | Token::LeftParen => self.next_call_fn(),
                    Token::Equal => self.next_assign(),
                    _ => panic!("Unexpected token: {:?}", self.peek_at(2).unwrap()),
                };
                self.clean_semicolon();
                to_return
            }
            token => panic!("Expected statement, got {:?}", token),
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
                Some(token) => panic!("Expected ',' or ')' but got {:?}", token),
                None => panic!("Unexpected end of input"),
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
                    _ => panic!("Invalid return type: {:?}", type_),
                };
            } else {
                panic!("Expected return type");
            }
        }

        self.advance();

        while let Some(tok) = self.peek() {
            if tok == Token::RightBrace {
                self.advance();
                break;
            }
            fn_body.push(self.get_next());
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

        let var_value: Expr = self.get_value_with_conditions_and_ops();

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
        let condition: Expr = self.get_value_with_conditions_and_ops();
        if self.advance().unwrap() != Token::LeftBrace {
            panic!("Expected '{}'", "{");
        }

        let mut then_branch: Vec<Node> = vec![];
        while let Some(tok) = self.peek() {
            if tok == Token::RightBrace {
                self.advance();
                break;
            }
            then_branch.push(self.get_next());
        }

        let mut else_branch: Vec<Node> = vec![];
        if self.peek() == Some(Token::Else) {
            self.advance();
            if self.peek().unwrap() != Token::LeftBrace {
                panic!("Expected '{}' but got {:?}", "{", self.peek().unwrap());
            }
            self.advance();
            while let Some(tok) = self.peek() {
                if tok == Token::RightBrace {
                    self.advance();
                    break;
                }

                else_branch.push(self.get_next());
            }
        } else if self.peek() == Some(Token::Elif) {
            else_branch = vec![self.next_if()];
        }

        Node::If {
            condition,
            then_branch,
            else_branch: if else_branch.len() == 0 {
                None
            } else {
                Some(else_branch)
            },
        }
    }

    fn next_while(&mut self) -> Node {
        self.advance();
        let condition: Expr = self.get_value_with_conditions_and_ops();
        if self.advance().unwrap() != Token::LeftBrace {
            panic!("Expected '{}'", "{");
        }

        let mut body: Vec<Node> = vec![];
        while let Some(tok) = self.peek() {
            if tok == Token::RightBrace {
                self.advance();
                break;
            }
            body.push(self.get_next());
        }

        Node::While { condition, body }
    }

    fn next_for(&mut self) -> Node {
        self.advance();
        let init_name: String = match self.advance().unwrap() {
            Token::Ident(name) => name,
            token => panic!("Expected variable name but got {:?}", token),
        };
        if self.advance().unwrap() != Token::Colon {
            panic!("Expected ':'");
        }
        let (init_immutable, init_type) = self.get_type();
        let init = Node::Var {
            name: init_name,
            type_: init_type,
            value: Expr::NoValue,
            immutable: init_immutable,
        };
        if self.advance().unwrap() != Token::In {
            panic!("Expected 'in'");
        }

        let range: Expr;
        match self.advance().unwrap() {
            Token::IntLit(start_value) => {
                if self.advance().unwrap() != Token::DoublePeriod {
                    panic!("Expected '..' in the int range");
                }
                match self.advance().unwrap() {
                    Token::IntLit(end_value) => range = Expr::Range(start_value, end_value),
                    token => panic!("Expected int literal but got {:?}", token),
                }
            }
            Token::Ident(name) => range = Expr::Ident(name),
            token => panic!("Expected int literal or variable name but got {:?}", token),
        }

        let mut body: Vec<Node> = vec![];
        if self.advance().unwrap() != Token::LeftBrace {
            panic!("Expected '{}'", "{");
        }
        while let Some(tok) = self.peek() {
            if tok == Token::RightBrace {
                self.advance();
                break;
            }
            body.push(self.get_next());
        }
        Node::For {
            var: Box::new(init),
            iterable: range,
            body,
        }
    }

    fn next_call_fn(&mut self) -> Node {
        let mut callee: Expr = Expr::NoValue;

        if self.peek_at(1).unwrap() == Token::Colon {
            callee = match self.advance().unwrap() {
                Token::Ident(name) => Expr::Ident(name),
                _ => panic!("Expected callee name"),
            };
            self.advance();
        }

        if self.peek_at(1).unwrap() != Token::LeftParen {
            panic!("Expected '(' but got {:?}", self.peek_at(1).unwrap());
        }

        let name: String = match self.advance().unwrap() {
            Token::Ident(name) => name,
            _ => panic!("Expected function name"),
        };

        self.advance();
        let args: Vec<Expr> = self.collect_args();
        Node::CallFn { callee, name, args }
    }

    fn next_assign(&mut self) -> Node {
        match self.advance().unwrap() {
            Token::Ident(name) => {
                if self.peek().unwrap() != Token::Equal {
                    panic!("Expected '=' got {:?}", self.peek().unwrap());
                }
                self.advance();
                let value: Expr = self.get_value_with_conditions_and_ops();
                Node::Assign { name, value }
            }
            token => panic!("Expected variable name but got {:?}", token),
        }
    }

    fn next_return(&mut self) -> Node {
        self.advance();
        let value: Expr = self.get_value_with_conditions_and_ops();
        self.clean_semicolon();
        Node::Return(value)
    }

    fn collect_args(&mut self) -> Vec<Expr> {
        let mut args: Vec<Expr> = vec![];
        loop {
            if self.peek().unwrap() == Token::RightParen {
                self.advance();
                break;
            }
            args.push(self.get_value_with_conditions_and_ops());
            match self.advance() {
                Some(Token::Comma) => {}
                Some(Token::RightParen) => break,
                Some(token) => panic!("Expected ',' or ')' but got {:?}", token),
                None => panic!("Unexpected end of input"),
            }
        }
        args
    }

    fn parse_or_condition(&mut self) -> Expr {
        let mut left: Expr = self.parse_and_condition();
        while self.peek() == Some(Token::Or) {
            self.advance();
            let right: Expr = self.parse_and_condition();
            left = Expr::Binary(Box::new(left), BinaryOp::Or, Box::new(right));
        }
        left
    }

    fn parse_and_condition(&mut self) -> Expr {
        let mut left: Expr = self.parse_not_condition();
        while self.peek() == Some(Token::And) {
            self.advance();
            let right: Expr = self.parse_not_condition();
            left = Expr::Binary(Box::new(left), BinaryOp::And, Box::new(right));
        }
        left
    }

    fn parse_not_condition(&mut self) -> Expr {
        if self.peek() == Some(Token::Not) {
            self.advance();
            let right: Expr = self.parse_not_condition();
            return Expr::Unary(UnaryOp::Not, Box::new(right));
        }
        self.parse_comparison_condition()
    }

    fn parse_comparison_condition(&mut self) -> Expr {
        let left = self.parse_primary_condition();

        let op = match self.peek() {
            Some(Token::DoubleEqual) => BinaryOp::Eq,
            Some(Token::NotEqual) => BinaryOp::Neq,
            Some(Token::GreaterThan) => BinaryOp::Gt,
            Some(Token::LessThan) => BinaryOp::Lt,
            Some(Token::GreaterEqual) => BinaryOp::Gte,
            Some(Token::LessEqual) => BinaryOp::Lte,
            _ => return left,
        };

        self.advance();
        let right = self.parse_primary_condition();
        Expr::Binary(Box::new(left), op, Box::new(right))
    }

    fn parse_primary_condition(&mut self) -> Expr {
        match self.advance() {
            Some(Token::LeftParen) => {
                let expr: Expr = self.parse_or_condition();
                if self.advance() != Some(Token::RightParen) {
                    panic!("Expected ')'");
                }
                expr
            }
            Some(_) => {
                self.pos -= 1;
                self.parse_minus_op()
            }
            _ => panic!("Expected primary expression"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Token;
    use crate::lexer::Token::Ident;
    use crate::parser::{BinaryOp, Expr, Node, Type};

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

    #[test]
    fn test_if_parse() {
        let mut parser_1 = crate::parser::Parser::new(vec![
            Token::If,
            Token::Ident(String::from("Test")),
            Token::DoubleEqual,
            Token::IntLit(1),
            Token::LeftBrace,
            Token::RightBrace,
            Token::Elif,
            Token::Ident(String::from("Test2")),
            Token::DoubleEqual,
            Token::IntLit(2),
            Token::LeftBrace,
            Token::RightBrace,
            Token::Else,
            Token::LeftBrace,
            Token::RightBrace,
        ]);
        assert_eq!(
            parser_1.next_if(),
            Node::If {
                condition: Expr::Binary(
                    Box::new(Expr::Ident(String::from("Test"))),
                    BinaryOp::Eq,
                    Box::new(Expr::Int(1))
                ),
                then_branch: vec![],
                else_branch: Some(vec![Node::If {
                    condition: Expr::Binary(
                        Box::new(Expr::Ident(String::from("Test2"))),
                        BinaryOp::Eq,
                        Box::new(Expr::Int(2))
                    ),
                    then_branch: vec![],
                    else_branch: Some(vec![]),
                }])
            }
        )
    }
}
