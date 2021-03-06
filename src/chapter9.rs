#[macro_use]
extern crate lazy_static;


use std::env;
use std::fmt;
use std::collections::{HashMap, VecDeque};

lazy_static!{
    static ref KeyWord: HashMap<&'static str, Token> = {
        let mut m = HashMap::new();
        m.insert("BEGIN", Token::BEGIN);
        m.insert("END", Token::END);
        m
    };
}

#[derive(Clone)]
enum Token{
    EOF,
    OP1(char),
    OP2(char),
    LP,
    RP,
    INTEGER(u64),
    UNARY(char),
    BEGIN,
    END,
    DOT,
    ASSIGN,
    SEMI,
    ID(String),
    Other,
}

struct Interpreter{
    text: Vec<char>,
    current_token: Token,
    idx: usize,
}

struct TreeNode{
    node: Token,
    sub_nodes: Vec<TreeNode>,
}

impl fmt::Debug for Token{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        match self{
            Token::INTEGER(n) => write!(f, "INTEGER: {}", n),
            Token::OP1(c) => write!(f, "operation: {}", c),
            Token::OP2(c) => write!(f, "operation: {}", c),
            Token::EOF => write!(f, "EOF"),
            Token::LP => write!(f, "("),
            Token::RP => write!(f, ")"),
            Token::UNARY(c) => write!(f, "UNARY: {}", c),
            Token::BEGIN => write!(f, "BEGIN"),
            Token::END => write!(f, "END"),
            Token::DOT => write!(f, "DOT"),
            Token::ID(s) => write!(f, "variable: {}", s),
            Token::ASSIGN => write!(f, "ASSIGN"),
            Token::SEMI => write!(f, "SEMI"),
            Token::Other => write!(f, "Other"),
        }
    }
}

impl PartialEq for Token{
    fn eq(&self, other: &Token) -> bool{
        match (self, other){
            (Token::EOF, Token::EOF) => true,
            (Token::OP1(_), Token::OP1(_)) => true,
            (Token::OP2(_), Token::OP2(_)) => true,
            (Token::LP, Token::LP) => true,
            (Token::RP, Token::RP) => true,
            (Token::INTEGER(_), Token::INTEGER(_)) => true,
            (Token::UNARY(_), Token::UNARY(_)) => true,
            (Token::BEGIN, Token::BEGIN) => true,
            (Token::END, Token::END) => true,
            (Token::DOT, Token::DOT) => true,
            (Token::ASSIGN, Token::ASSIGN) => true,
            (Token::SEMI, Token::SEMI) => true,
            (Token::ID(_), Token::ID(_)) => true,
            (Token::Other, Token::Other) => true,
            _ => false,
        }
    }
}


impl Interpreter{
    pub fn from(s: &String) -> Self{
        let mut a = Interpreter{text: s.chars().collect(), current_token: Token::EOF, idx: 0};
        a.get_next_token();
        a
    }

    pub fn _id(&mut self) -> Token{
        let mut s: String = String::new();

        while self.idx < self.text.len(){
            match self.text[self.idx]{
                c @ 'a'...'z' | c @ 'A'...'Z' | c @ '0'...'9' => {
                    s.push(c);
                    self.idx += 1;
                },
                _ => break,
            }
        }
        match KeyWord.get(s.as_str()){
            Some(value) => value.clone(),
            None => Token::ID(s),
        }
    }

    pub fn get_digits(&mut self) -> u64{
        let mut ret: u64 = self.text[self.idx] as u64 - '0' as u64;
        self.idx += 1;
        while self.idx < self.text.len(){
            match self.text[self.idx]{
                c @ '0' ... '9' => {
                    ret = ret * 10 + c as u64 - '0' as u64;
                    self.idx += 1;
                },
                _ => break,
            }
        }
        ret
    }

    pub fn get_next_token(&mut self) -> Token{
        let mut ret: Token = Token::EOF;
        if self.idx >= self.text.len() {self.current_token = ret.clone(); return ret;}
        ret = match self.text[self.idx]{
                'a' ... 'z' | 'A' ... 'Z' => self._id(),
                ':' => {
                    if self.idx + 1 < self.text.len() && self.text[self.idx + 1] == '='{
                        self.idx += 2;
                        Token::ASSIGN
                    }else{
                        panic!("error in with char ':', missing '=' ");
                    }
                },
                ';' => {
                    self.idx += 1;
                    Token::SEMI
                },
                '.' => {
                    self.idx += 1;
                    Token::DOT
                },
                '0'..='9' => Token::INTEGER(self.get_digits()),
                c @ '+' | c @ '-' => {self.idx += 1; Token::OP1(c)},
                c @ '*' | c @ '/' => {self.idx += 1; Token::OP2(c)},
                '(' => {self.idx += 1; Token::LP},
                ')' => {self.idx += 1; Token::RP},
                ' ' | '\n' => {
                    self.idx += 1;
                    while self.idx < self.text.len(){
                        match self.text[self.idx]{
                            ' ' | '\n' => self.idx += 1,
                            _ => break,
                        }
                    }
                    self.get_next_token()
                },
                e => panic!(format!("error in chars {}({})", e, self.idx)),
            };
        self.current_token = ret.clone();
        ret
    }

    fn eat(&mut self, token: Token){
        if &self.current_token == &token{
            self.get_next_token();
        }else{
            panic!(format!("error in fn eat, current token: {:?}, token: {:?}", self.current_token, token));
        }
    }

    fn program1(&mut self) -> TreeNode{
        let node = self.compound_statement();
        self.eat(Token::DOT);
        TreeNode{node: Token::Other, sub_nodes: vec![node]}
    }

    fn compound_statement(&mut self) -> TreeNode{
        self.eat(Token::BEGIN);
        let nodes: Vec<TreeNode> = self.statement_list();
        self.eat(Token::END);
        TreeNode{node: Token::Other, sub_nodes: nodes}
    }

    fn statement_list(&mut self) -> Vec<TreeNode>{
        let mut nodes: Vec<TreeNode> = Vec::new();
        nodes.push(self.statement());
        while self.current_token == Token::SEMI{
            self.eat(Token::SEMI);
            nodes.push(self.statement());
        }

        if self.current_token == Token::ID('a'.to_string()){
            panic!(format!("error in fn statement list, current token: {:?}", self.current_token));
        }

        nodes
    }

    fn statement(&mut self) -> TreeNode{
        let node = match &self.current_token{
            Token::BEGIN => self.compound_statement(),
            Token::ID(v) => self.assignment_statement(),
            _ => self.empty(),
        };
        node
    }

    fn assignment_statement(&mut self) -> TreeNode{
        let left = self.variable();
        let token = self.current_token.clone();
        self.eat(Token::ASSIGN);
        let right = self.expr();
        let node = TreeNode{node: token, sub_nodes: vec![left, right]};
        node
    }

    fn variable(&mut self) -> TreeNode{
        let node = TreeNode{node: self.current_token.clone(), sub_nodes: vec![]};
        self.get_next_token();
        node
    }

    fn empty(&mut self) -> TreeNode{
        TreeNode{node: Token::Other, sub_nodes: vec![]}
    }


    fn factor(&mut self) -> TreeNode{
        let token = self.current_token.clone();
        match token{
            Token::OP1(c) => {
                self.get_next_token();
                TreeNode{node: Token::UNARY(c), sub_nodes: vec![self.factor()]}
            },
            Token::INTEGER(n) => {
                self.get_next_token();
                TreeNode{node: token, sub_nodes: vec![]}
            }
            Token::LP => {
                self.get_next_token();
                let node = self.expr();
                self.eat(Token::RP);
                node
            }
            Token::ID(c) =>{
                self.variable()
            }
            _ => panic!(format!("error in fn factor Token::ID branch, token: {:?}", token)),
        }
    }

    fn term(&mut self) -> TreeNode{
        let mut node = self.factor();
       
        loop{
            match &self.current_token{
                Token::OP2(_) => {
                    let token = self.current_token.clone();
                    self.get_next_token();
                    node = TreeNode{node: token, sub_nodes: vec![node, self.factor()]};
                }
                _ => break,
            }
        }
        node
    }

    fn expr(&mut self) -> TreeNode{
        let mut node = self.term();
       
        loop{
            match &self.current_token{
                Token::OP1(_) => {
                    let token = self.current_token.clone();
                    self.get_next_token();
                    node = TreeNode{node: token, sub_nodes: vec![node, self.term()]};
                }
                _ => break,
            }
        }
        node
    }

    pub fn parse(&mut self) -> TreeNode{
        let node = self.program1();
        if self.current_token != Token::EOF{
            panic!(format!("error in parse: do not found EOF in the end. current token: {:?}", self.current_token));
        }
        self.idx = 0;
        self.get_next_token();
        node
    }   
}

impl fmt::Debug for TreeNode{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        let mut q: VecDeque<&TreeNode> = VecDeque::new();
        q.push_back(&self);

        let mut pre_level_num;
        let mut this_level_num = 1;

        let mut ret: String = String::new();

        while !q.is_empty(){
            pre_level_num = this_level_num;
            this_level_num = 0;
            for i in 0..pre_level_num{
                let node = q.pop_front().unwrap();
                match node.node{
                    Token::ID(_) | Token::INTEGER(_) => 
                      ret += format!("  |{:?}(0)|  ", node.node).as_str(),
                    _ => {
                        ret += format!("  |{:?}({})|  ", node.node, node.sub_nodes.len()).as_str();
                        for n in node.sub_nodes.iter(){
                            q.push_back(n);
                            this_level_num += 1;
                        }
                    }
                }
            }
            ret.push('\n');
        }

        write!(f, "{}", ret)
    }
}


struct Visit{
    global_scope: HashMap<String, i64>,
}

impl Visit{
    
    fn new() -> Self{
        Visit{global_scope: HashMap::new()}
    }

    fn visit(&mut self, root: &TreeNode){
        match &root.node{
            Token::ASSIGN => {
                let var_name = self.vist_Var(&root.sub_nodes[0]);
                self.global_scope.insert(var_name, self.visit_value(&root.sub_nodes[1]));
            },
            Token::Other => {
                for stat in root.sub_nodes.iter(){
                    self.visit(stat);
                }
            },
            _ => panic!(format!("error in fn visit. current node: {:?}", root.node)),
        }
    }

    fn visit_value(&self, root: &TreeNode) -> i64{
        match &root.node{
            Token::ID(s) => self.global_scope[s],
            Token::INTEGER(n) => *n as i64,
            Token::UNARY('+') => self.visit_value(&root.sub_nodes[0]),
            Token::UNARY('-') => -self.visit_value(&root.sub_nodes[0]),
            Token::OP1('+') => {
                self.visit_value(&root.sub_nodes[0]) + self.visit_value(&root.sub_nodes[1])
            }
            Token::OP1('-') => {
                self.visit_value(&root.sub_nodes[0]) - self.visit_value(&root.sub_nodes[1])
            }
            Token::OP2('*') => {
                self.visit_value(&root.sub_nodes[0]) * self.visit_value(&root.sub_nodes[1])
            }
            Token::OP2('/') => {
                self.visit_value(&root.sub_nodes[0]) / self.visit_value(&root.sub_nodes[1])
            }
            _ => panic!(format!("error in fn visit_value. current node: {:?}", root.node)),
        }
    }

    pub fn vist_Var(&mut self, root: &TreeNode) -> String{
        match &root.node{
            Token::ID(s) => s.clone(),
            _ => panic!(format!("error in fn vist_Var. current node: {:?}", root.node)),
        }
    }
}

