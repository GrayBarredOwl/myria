use std::sync::OnceLock;

use crate::ast::Expression;
use crate::gen::{self, keywords, operators, Operator};
use crate::obj::Object;

#[derive(PartialEq, Debug, Clone)]
pub enum TokenType {
    Int(i64),
    Float(f64),
    Char(char),
    Str(String),
    Id(String),
    Keyword(gen::Keyword),
    Operator(gen::Operator),
}

impl TokenType {
    pub fn is_value(&self) -> bool {
        matches!(
            self,
            Self::Int(_) | Self::Float(_) | Self::Str(_) | Self::Char(_) | Self::Id(_)
        )
    }
    pub fn is_unary_op(&self) -> bool {
        matches!(
            self,
            Self::Operator(Operator::Minus) | Self::Operator(Operator::Not)
        )
    }

    pub fn is_binary_op(&self) -> bool {
        type Op = Operator;
        let o = match self {
            Self::Operator(Operator::Not) => return false,
            Self::Operator(o) => o,
            _ => return false,
        };

        matches!(
            *o,
            Op::And
                | Op::Assign
                | Op::Dot
                | Op::Equals
                | Op::Gr
                | Op::GrEq
                | Op::Lt
                | Op::LtEq
                | Op::Minus
                | Op::NEqual
                | Op::Or
                | Op::Plus
                | Op::Slash
                | Op::Star
                | Op::PlusEq
                | Op::MinusEq
                | Op::StarEq
                | Op::SlashEq
        )
    }
    pub fn is_semi(&self) -> bool {
        matches!(self, TokenType::Operator(Operator::Semicolon))
    }

    pub fn to_value(&self) -> Option<Expression> {
        if self.is_value() {
            Some(match self {
                Self::Int(n) => Object::make_int(*n).make_expr(),
                Self::Float(n) => Object::make_float(*n).make_expr(),
                Self::Char(c) => Object::make_char(*c).make_expr(),
                Self::Id(name) => Expression::Variable(name.clone()),
                _ => panic!(),
            })
        } else {
            None
        }
    }
    pub fn to_op(&self) -> Option<Operator> {
        match self {
            Self::Operator(o) => Some(*o),
            _ => None,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Meta {
    pub line_number: usize,
}

#[derive(Clone)]
pub struct Token {
    pub info: TokenType,
    pub metadata: Meta,
}
impl Default for Token {
    fn default() -> Self {
        Self {
            info: TokenType::Operator(Operator::Semicolon),
            metadata: Meta { line_number: 0 },
        }
    }
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Token({:?})", self.info)
    }
}

impl Token {
    fn new(ttype: TokenType, tmeta: Meta) -> Self {
        Token {
            info: ttype,
            metadata: tmeta,
        }
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.info == other.info
    }
}

pub struct Lexer<'a> {
    current: usize,
    cur_line: usize,
    string: &'a str,
}
impl<'a> Lexer<'a> {
    pub fn new(string: &'a str) -> Self {
        Self {
            current: 0,
            cur_line: 1,
            string,
        }
    }
    pub fn tokenize(mut self) -> Vec<Token> {
        let mut vec = vec![];

        while self.can_peek() {
            let c = self.peek();

            if c.is_whitespace() {
                if c == '\n' {
                    vec.push(self.make_tok(TokenType::Operator(Operator::Semicolon)));
                }
                self.resolve_whitespace();
                continue;
            } else if c == '#' {
                self.resolve_comment();
                continue;
            }
            let t = if c.is_numeric() {
                self.resolve_number()
            } else if Self::op_first_chars().contains(&c) {
                self.resolve_op()
            } else if c == '\'' {
                self.resolve_char()
            } else if c == '\"' {
                self.resolve_string()
            } else if c.is_alphabetic() || c == '_' {
                self.resolve_idkw()
            } else {
                panic!("(Line {}) Unrecognized token", self.cur_line);
            };
            vec.push(t);
        }

        vec
    }
    fn can_peek(&self) -> bool {
        self.can_peek_ahead(0)
    }
    fn can_peek_ahead(&self, n: usize) -> bool {
        self.current + n < self.string.chars().count()
    }
    fn peek_ahead(&self, n: usize) -> char {
        self.string.chars().nth(self.current + n).unwrap()
    }
    fn peek(&self) -> char {
        self.peek_ahead(0)
    }
    fn consume(&mut self) -> char {
        let c = self.peek();
        self.current += 1;
        c
    }
    fn unconsume(&mut self) {
        assert!(self.current > 0);
        self.current -= 1;
    }
    fn resolve_whitespace(&mut self) {
        assert!(self.can_peek() && self.peek().is_whitespace());
        let w = self.consume();
        if w == '\n' {
            self.cur_line += 1;
        }
    }
    fn resolve_comment(&mut self) {
        assert!(self.can_peek() && self.peek() == '#');
        while self.can_peek() && self.consume() != '\n' {}
        self.cur_line += 1;
    }
    fn resolve_number(&mut self) -> Token {
        assert!(self.can_peek() && self.peek().is_numeric());
        let mut num_dots = 0;
        let start = self.current;

        while self.can_peek() {
            let n = self.consume();
            if n.is_whitespace() || (Self::op_first_chars().contains(&n) && n != '.') {
                self.unconsume();
                break;
            } else if n == '.' {
                num_dots += 1;
                if num_dots > 1 {
                    panic!(
                        "(Line {}) Number can not contain multiple periods",
                        self.cur_line
                    );
                }
            } else if !n.is_numeric() {
                panic!(
                    "(Line {}) Number can only contain 0-9 and period",
                    self.cur_line
                );
            }
        }

        if num_dots == 0 {
            let i = self.string[start..self.current].parse().unwrap();
            self.make_tok(TokenType::Int(i))
        } else {
            let f = self.string[start..self.current].parse().unwrap();
            self.make_tok(TokenType::Float(f))
        }
    }
    fn resolve_op(&mut self) -> Token {
        assert!(self.can_peek() && Self::op_first_chars().contains(&self.peek()));
        let start = self.current;
        let _first = self.consume();
        if !self.can_peek() {
            return self.make_tok(TokenType::Operator(
                *operators().get(&self.string[start..]).unwrap(),
            ));
        }
        let second = self.consume();
        if !matches!(second, '=' | '&' | '|') {
            self.unconsume();
            return self.make_tok(TokenType::Operator(
                *operators().get(&self.string[start..self.current]).unwrap(),
            ));
        }

        self.make_tok(TokenType::Operator(
            *operators().get(&self.string[start..self.current]).unwrap(),
        ))
    }
    fn resolve_char(&mut self) -> Token {
        assert!(self.can_peek() && self.peek() == '\'');
        let _open = self.consume();
        if !self.can_peek_ahead(1) {
            panic!("(Line {})Unclosed single quote!", self.cur_line);
        }
        let c = self.consume();
        let _close = self.consume();
        self.make_tok(TokenType::Char(c))
    }
    fn resolve_string(&mut self) -> Token {
        assert!(self.can_peek() && self.peek() == '\"');
        let _open = self.consume();
        let start = self.current;
        while self.can_peek() {
            let c = self.consume();
            if c == '\"' {
                break;
            } else if c == '\n' {
                panic!("(Line {}) Unclosed string", self.cur_line);
            }
        }
        self.unconsume();
        if self.peek() != '\"' {
            panic!("(Line {}) Unclosed string", self.cur_line);
        }
        let s = self.string[start..(self.current)].to_string();
        self.consume();
        self.make_tok(TokenType::Str(s))
    }
    fn resolve_idkw(&mut self) -> Token {
        assert!(self.can_peek() && (self.peek().is_alphabetic() || self.peek() == '_'));
        let start = self.current;
        self.consume();
        while self.can_peek() {
            let c = self.consume();
            if c.is_alphanumeric() || c == '_' {
            } else {
                self.unconsume();
                break;
            }
        }
        let id = &self.string[start..self.current];
        if let Some(kw) = keywords().get(id) {
            self.make_tok(TokenType::Keyword(*kw))
        } else {
            self.make_tok(TokenType::Id(id.into()))
        }
    }
    fn make_tok(&self, tt: TokenType) -> Token {
        Token::new(
            tt,
            Meta {
                line_number: self.cur_line,
            },
        )
    }
    fn op_first_chars() -> &'static Vec<char> {
        static VEC: OnceLock<Vec<char>> = OnceLock::new();
        VEC.get_or_init(|| {
            operators()
                .keys()
                .map(|k| k.chars().next().unwrap())
                .collect()
        })
    }
}
